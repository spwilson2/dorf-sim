pub mod error;
pub mod grid;
pub mod on_exit;
pub mod rect2d;
pub mod transform;

pub use self::error::*;
pub use self::grid::*;
pub use self::on_exit::*;
pub use self::rect2d::*;
pub use self::transform::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[allow(non_snake_case)]
pub mod Color {
    use super::RGB;
    pub const RED: RGB = RGB::new(255, 0, 0);
}

impl RGB {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PaintChar {
    pub c: char,
    pub rgb: Option<RGB>,
}

impl Default for PaintChar {
    fn default() -> Self {
        Self { c: ' ', rgb: None }
    }
}
