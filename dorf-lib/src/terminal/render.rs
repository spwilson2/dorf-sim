use crate::prelude::*;
use std::{
    cmp::{max, min, Ordering},
    collections::VecDeque,
};

use super::{camera::TerminalCamera2D, display::TerminalDisplayBuffer};

/// This plugin is responsible for providing Components which can be rendered
/// down onto a terminal screen and then painted.  Render logic is super simple:
/// The TransformTexture with the highest z value will be painted.
#[derive(Default)]
pub struct TerminalRenderPlugin();
impl Plugin for TerminalRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(render);
    }
}

/// Local cache for the rendering function. Rather than needing to allocate a
/// new Vec, each time keep one static.
#[derive(Default)]
struct RenderCache {
    z_sort_cache: Vec<CharTextureTransform>,
}

fn render(
    mut local: Local<RenderCache>,
    changed: Query<(&CharTexture, &Transform2D), Or<(Changed<Transform2D>, Changed<CharTexture>)>>,
    all_textures: Query<(&CharTexture, &Transform2D)>,
    camera: Res<TerminalCamera2D>,
    mut display_buf: ResMut<TerminalDisplayBuffer>,
) {
    if changed.is_empty() && !display_buf.is_changed() && !camera.is_changed() {
        return;
    }
    let buf_width = display_buf.width;
    let buf_height = display_buf.height;

    #[cfg(debug_assertions)]
    if buf_width < camera.dim().x as u16 || buf_height < camera.dim().y as u16 {
        // This is an acceptable race condition. Basically we may render down at
        // the same time as the terminal is being resized, so only the camera
        // may have been resized but not the terminal backing buffer.
        log::warn!(
            "Camera dimmensions larger than terminal ({:?}) > {:?}",
            (camera.dim().x as usize, camera.dim().y as usize),
            (buf_width, buf_height)
        );
    }

    // Keep a cache of all transforms in increasing order of their z level.
    // This way when we naively print every single transform, the transforms
    // with the highest z level will be printed last and on top.
    local.z_sort_cache.clear();
    local
        .z_sort_cache
        .extend(
            all_textures
                .iter()
                .map(|(texture, transform)| CharTextureTransform {
                    texture: texture.clone(),
                    transform: transform.clone(),
                }),
        );
    local.z_sort_cache.sort_by(|l, r| {
        r.transform
            .z_lvl()
            .partial_cmp(&l.transform.z_lvl())
            .unwrap()
    });

    // Stash the bounds/dimensions to paint, we won't need to pain anything outside the camera.
    let camera_rec = Rect2D::from_transform2d(&camera.transform);

    // Start by clearing the frame buffer, render will completely refill it.
    // Then begin processing each texture.
    display_buf.reinit();
    for text_transform in local.z_sort_cache.iter() {
        // Iterate through all textures,
        let overlap = camera_rec.intersect(Rect2D::from_transform2d(&text_transform.transform));
        if overlap.is_empty() {
            // If no overlayp, just continue to next
            continue;
        }

        // Move the starting point to be relative to the camera (i.e. the buffer
        // is relative to the camera so x0=camera.x, y0=camera.y)
        let start = overlap.min - camera_rec.min;
        // Again, move point to be relative to the camera, and then trim any
        // extra past the max buffer size.
        let end = (overlap.max - camera_rec.min).min(IVec2 {
            x: buf_width as i32,
            y: buf_height as i32,
        });

        // Iterate only through the sections that we're updating, and write.
        for row in start.y..end.y {
            for col in start.x..end.x {
                let tile = display_buf
                    .c_vec
                    .get_mut((col + row * buf_width as i32) as usize)
                    .unwrap();
                if *tile == ' ' {
                    *tile = text_transform.texture.texture;
                }
            }
        }
    }
}
