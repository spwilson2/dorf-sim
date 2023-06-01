use crate::prelude::*;
use std::{
    cmp::{max, min, Ordering},
    collections::VecDeque,
};

/// This plugin is responsible for providing Components which can be rendered down onto a terminal screen and then painted.
/// Render logic is super simple: The TransformTexture with the highest z value will be painted.
use super::{camera::TerminalCamera2D, display::TerminalDisplayBuffer};

#[derive(Bundle, Clone)]
pub struct CharTextureTransform {
    texture: CharTexture,
    transform: Transform2D,
}

/// Simple texture on top of transform
#[derive(Component, Debug, Clone)]
pub struct CharTexture {
    pub texture: char,
}

#[derive(Component)]
pub struct CharPaintGrid {
    grid: Grid2D<char>,
}

#[derive(Default)]
pub struct TerminalRenderPlugin();

impl Plugin for TerminalRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(render);
    }
}

/// Local cache for the rendering function. Rather than needing to allocate a new Vec, each time keep one static.
#[derive(Default)]
struct RenderCache {
    buf: Vec<Tile>,
    sort_cache: Vec<CharTextureTransform>,
    width: u16,
    depth: u16,
}

struct Tile {
    texture: char,
    z_depth: f32,
}

fn advance_by<T>(mut itr: impl Iterator<Item = T>, n: usize) -> Result<(), usize> {
    for i in 0..n {
        itr.next().ok_or(i)?;
    }
    Ok(())
}

// To normalize x:
// div_x = max_value_x - min_value_x
// f(x) = (x - min_value_x) / div_x
#[inline]
fn normalize_point(point: Vec2, max_point: Vec2, min_point: Vec2) -> Vec2 {
    let div = max_point - min_point;
    (point - min_point) / div
}

#[inline]
fn normalized_point_to_tile(point: Vec2, width: u16, height: u16) -> (u16, u16) {
    (
        (point.x * width as f32) as u16,
        (point.y * height as f32) as u16,
    )
}

fn render(
    mut cache: Local<RenderCache>,
    changed: Query<(&CharTexture, &Transform2D), Changed<Transform2D>>,
    query: Query<(&CharTexture, &Transform2D)>,
    camera: ResMut<TerminalCamera2D>,
    mut display_buf: ResMut<TerminalDisplayBuffer>,
) {
    if changed.is_empty() && !display_buf.is_changed() && !camera.is_changed() {
        return;
    }
    let buf_width = display_buf.width;
    let buf_height = display_buf.height;
    if camera.settings().autoresize() {
        // FIXME?Maybe todo? Not sure if old impl was actually broken.
        //camera.set_dim(UVec2::new(buf_width as u32, buf_height as u32));

        // camera.dim().x = buf_width as u32;
        // camera.dim().y = buf_height as u32;
    }

    // Get bounds/dimensions to paint, we won't need to pain anything outside bounds.
    let camera_rec = Rect2D::from_transform2d(&camera.transform);

    cache.sort_cache.clear();
    cache.sort_cache.extend(
        query
            .iter()
            .map(|(texture, transform)| CharTextureTransform {
                texture: texture.clone(),
                transform: transform.clone(),
            }),
    );
    cache.sort_cache.sort_by(|l, r| {
        r.transform
            .z_lvl()
            .partial_cmp(&l.transform.z_lvl())
            .unwrap()
    });

    // Start by clearing the frame buffer, render will completely fill it.
    display_buf.c_vec.clear();
    display_buf
        .c_vec
        .resize((buf_height * buf_width) as usize, ' ');

    if buf_width < camera.dim().x as u16 || buf_height < camera.dim().y as u16 {
        log::warn!(
            "Camera dimmensions larger than terminal ({:?}) > {:?}",
            (camera.dim().x as usize, camera.dim().y as usize),
            (buf_width, buf_height)
        );
    }

    // For each tile keep the texture of the max z.
    // (Obviously this is the naive and super inefficient way to do this, but I don't know anything about SIMD/GPU optimizations for layering textures...)
    for text_transform in cache.sort_cache.iter() {
        // Iterate through all textures,
        let overlap = camera_rec.intersect(Rect2D::from_transform2d(&text_transform.transform));
        if overlap.is_empty() {
            continue;
        }

        let start_x;
        let start_y;
        let end_x;
        let end_y;
        let tile_min = overlap.min - camera_rec.min;
        let tile_max = overlap.max - camera_rec.min;
        (start_x, start_y) = (tile_min.x as u16, tile_min.y as u16);
        //  Cap them to the buffer size.
        (end_x, end_y) = (
            min(tile_max.x as u16, buf_width),
            min(tile_max.y as u16, buf_height),
        );

        // Iterate through the sections that we're actually updating
        for row in start_y..end_y {
            for col in start_x..end_x {
                let tile = display_buf
                    .c_vec
                    .get_mut((col + row * buf_width) as usize)
                    .unwrap();
                if *tile == ' ' {
                    *tile = text_transform.texture.texture;
                }
            }
        }
    }
}

#[test]
fn test_normalize_point() {
    let min = Vec2::new(0.0, 0.0);
    let max = Vec2::new(10.0, 10.0);

    let norm_min = normalize_point(min, max, min);
    assert_eq!(norm_min, Vec2::new(0.0, 0.0));
    let norm_max = normalize_point(max, max, min);
    assert_eq!(norm_max, Vec2::new(1.0, 1.0));
    assert_eq!(
        normalize_point(Vec2::new(5.0, 5.0), max, min),
        Vec2::new(0.5, 0.5)
    );

    let min = Vec2::new(0.0, 0.0);
    let max = Vec2::new(10.0, 20.0);
    assert_eq!(
        normalize_point(Vec2::new(5.0, 5.0), max, min),
        Vec2::new(0.5, 0.25)
    );
}

#[test]
fn test_normalize_point_to_tile() {
    assert_eq!(
        normalized_point_to_tile(Vec2::new(0.0, 0.0), 10, 10),
        (0u16, 0u16)
    );
    assert_eq!(
        normalized_point_to_tile(Vec2::new(1.0, 1.0), 10, 10),
        (10u16, 10u16)
    );
    assert_eq!(
        normalized_point_to_tile(Vec2::new(0.5, 0.5), 10, 10),
        (5u16, 5u16)
    );

    assert_eq!(
        normalized_point_to_tile(Vec2::new(0.5, 0.5), 10, 20),
        (5u16, 10u16)
    );
}
