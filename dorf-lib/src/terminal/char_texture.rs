use crate::prelude::*;

#[derive(Bundle, Clone, Debug)]
pub struct CharTextureTransform {
    pub texture: CharTexture,
    pub transform: Transform2D,
}
/// Simple texture on top of transform
#[derive(Component, Debug, Clone, PartialEq)]
pub struct CharTexture {
    pub c: char,
    pub rgb: Option<RGB>,
}
impl Default for CharTexture {
    fn default() -> Self {
        Self { c: ' ', rgb: None }
    }
}

impl CharTexture {
    pub fn new(texture: char, rgb: RGB) -> Self {
        Self {
            c: texture,
            rgb: Some(rgb),
        }
    }
    pub fn from_char(texture: char) -> Self {
        Self {
            c: texture,
            rgb: None,
        }
    }
}
