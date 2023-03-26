
use std::{collections::VecDeque, cmp::Ordering};

use bevy::math::Vec3Swizzles;

/// This plugin is responsible for providing Components which can be rendered down onto a terminal screen and then painted.
/// Render logic is super simple: The TextureRect with the highest z value will be painted.

use crate::prelude::*;

use super::display::{TerminalDisplayBuffer, self};

#[derive(Resource, Default)]
pub struct TerminalCamera2d {
    pub dim: Vec2,
    pub loc: Vec3,
}

#[derive(Component, Clone)]
pub struct TextureRect {
    pub texture: char,
    pub dim: Vec2,
    pub loc: Vec2,
    pub loc_z: f32,
}

#[derive(Resource)]
struct RenderSettings {
    // TODO: Scale of render? This might be better suited for camera? Idk if I need that object for this tbh.
    scale: f32,
}

#[derive(Default)]
pub struct TerminalRenderPlugin();

impl Plugin for TerminalRenderPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(TerminalCamera2d::default())
        .add_system(render);
    }
}

/// Local cache for the rendering function. Rather than needing to allocate a new Vec, each time keep one static. 
#[derive(Default)]
struct RenderCache {
    buf: Vec<Tile>,
    sort_cache: Vec<TextureRect>,
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
    ((point.x * width as f32) as u16, (point.y * height as f32) as u16)
}

fn render(mut cache: Local<RenderCache>, query: Query<&TextureRect>, camera: Res<TerminalCamera2d>, mut display_buf: ResMut<TerminalDisplayBuffer>) {

    // TODO: Using the camera dimensions we will normalize onto the RenderCache, and then finalize by writing to the TerminalDisplayBuffer.

    // Get bounds/dimensions to paint, we won't need to pain anything outside bounds. 
    //let top_left = camera.loc.xy() - (camera.dim / 2.0);
    //let bottom_right = camera.loc.xy() + (camera.dim / 2.0);
    //let bottom_left = bottom_right - Vec2{x:camera.dim.x, y:0.0};
    //let top_right = top_left + Vec2{x:camera.dim.x, y:0.0};
    let camera_rec = Rect::from_center_size(camera.loc.xy(), camera.dim);

    // Normalize dimmensions down onto tiles so we can pain individual points.
    let hight_flt_per_px = display_buf.0.height as f32 / camera.dim.x;
    let width_flt_per_px = display_buf.0.width as f32 / camera.dim.y;

    cache.sort_cache.clear();
    cache.sort_cache.extend(query.iter().map(|rect_ref| (*rect_ref).clone()));
    cache.sort_cache.sort_by(|l, r| r.loc_z.partial_cmp(&l.loc_z).unwrap());

    let buf_width = display_buf.0.width;
    let buf_height = display_buf.0.height;
    // Start by clearing the frame buffer, render will completely fill it.
    display_buf.0.buf.clear();
    display_buf.0.buf.resize((buf_height * buf_width) as usize, ' ');

    // For each tile keep the texture of the max z.
    // (Obviously this is the naive and super inefficient way to do this, but I don't know anything about SIMD/GPU optimizations for layering textures...)
    for texture in cache.sort_cache.iter() {

        // Iterate through all textures, 
        let overlap = camera_rec.intersect(Rect::from_center_size(texture.loc, texture.dim));
        if overlap.is_empty() {
            continue;
        }
        // Check the zdepth for all tiles
        //let start_x = overlap.min.x - camera_rec.min.x;
        //let end_x = overlap.max.x - camera_rec.max.x;
        //let start_y = overlap.max.y - camera_rec.max.y;
        //let end_y = overlap.min.y - camera_rec.min.y;

        let norm_min = normalize_point(overlap.min, camera_rec.max, camera_rec.min);
        let norm_max = normalize_point(overlap.max, camera_rec.max, camera_rec.min);
        let (start_x, start_y) = normalized_point_to_tile(norm_min, buf_width, buf_height);
        let (end_x, end_y) = normalized_point_to_tile(norm_max, buf_width, buf_height);
        let (paint_width, _paint_height) = ((end_x - start_x), (end_y - start_y));

        // Iterate through the sections that we're actually updating
        for row in start_y..end_y {
            for col in start_x..end_x {
                let tile = display_buf.0.buf.get_mut((col + row*buf_width) as usize).unwrap();
                if *tile == ' ' {
                    *tile = texture.texture;
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
    assert_eq!(normalize_point(Vec2::new(5.0, 5.0), max, min), Vec2::new(0.5, 0.5));

    let min = Vec2::new(0.0, 0.0);
    let max = Vec2::new(10.0, 20.0);
    assert_eq!(normalize_point(Vec2::new(5.0, 5.0), max, min), Vec2::new(0.5, 0.25));
}

#[test]
fn test_normalize_point_to_tile() {
    assert_eq!(normalized_point_to_tile(Vec2::new(0.0, 0.0), 10, 10), (0u16, 0u16));
    assert_eq!(normalized_point_to_tile(Vec2::new(1.0, 1.0), 10, 10), (10u16, 10u16));
    assert_eq!(normalized_point_to_tile(Vec2::new(0.5, 0.5), 10, 10), (5u16, 5u16));

    assert_eq!(normalized_point_to_tile(Vec2::new(0.5, 0.5), 10, 20), (5u16, 10u16));
}