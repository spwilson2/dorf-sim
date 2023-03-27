use crate::prelude::*;

use super::input::TerminalResize;

#[derive(Default)]
pub struct TerminalCamera2dPlugin();

#[derive(Default)]
pub struct CameraResized(pub Vec2);

impl Plugin for TerminalCamera2dPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerminalCamera2d::default())
            .add_startup_system(init_camera_autosize)
            .add_event::<CameraResized>()
            .add_system(handle_terminal_resize);
    }
}

fn init_camera_autosize(
    mut camera: ResMut<TerminalCamera2d>,
    mut camera_event_writer: EventWriter<CameraResized>,
) {
    if camera.settings_ref().autoresize() {
        let term_size = crossterm::terminal::size().unwrap();
        let update = Vec2::new(term_size.0 as f32, term_size.1 as f32);
        camera.set_dim(update);
        camera_event_writer.send(CameraResized(update));
    }
}

fn handle_terminal_resize(
    mut camera: ResMut<TerminalCamera2d>,
    mut resize_reader: EventReader<TerminalResize>,
    mut camera_event_writer: EventWriter<CameraResized>,
) {
    if !camera.settings_ref().autoresize() {
        return;
    }
    if let Some(resize) = resize_reader.iter().last() {
        let update = Vec2::new(resize.width as f32, resize.height as f32);
        if update != camera.dim() {
            camera.set_dim(update);
            camera_event_writer.send(CameraResized(update));
        }
    }
}

#[derive(Resource, Default)]
pub struct TerminalCamera2d {
    dim: Vec2,
    loc: Vec3,
    settings: TerminalCamera2dSettings,
}

impl TerminalCamera2d {
    pub fn new(dim: Vec2, loc: Vec3) -> Self {
        Self {
            dim,
            loc,
            ..Default::default()
        }
    }
    pub fn width(&self) -> f32 {
        self.dim.x
    }
    pub fn height(&self) -> f32 {
        self.dim.y
    }
    pub fn set_width(&mut self, width: f32) {
        self.dim.x = width
    }
    pub fn set_height(&mut self, height: f32) {
        self.dim.y = height
    }
    pub fn x(&self) -> f32 {
        self.loc.x
    }
    pub fn y(&self) -> f32 {
        self.loc.y
    }
    pub fn z(&self) -> f32 {
        self.loc.z
    }
    pub fn move_by(&mut self, vec: Vec3) {
        self.loc += vec;
    }
    pub fn move_x(&mut self, x: f32) {
        self.loc.x += x
    }
    pub fn move_y(&mut self, y: f32) {
        self.loc.y += y
    }
    pub fn move_z(&mut self, z: f32) {
        self.loc.z += z
    }
    pub fn set_x(&mut self, x: f32) {
        self.loc.x = x
    }
    pub fn set_y(&mut self, y: f32) {
        self.loc.y = y
    }
    pub fn set_z(&mut self, z: f32) {
        self.loc.z = z
    }
    pub fn settings_ref(&self) -> &TerminalCamera2dSettings {
        &self.settings
    }

    pub fn set_loc(&mut self, loc: Vec3) {
        self.loc = loc;
    }

    pub fn loc(&self) -> Vec3 {
        self.loc
    }

    pub fn dim(&self) -> Vec2 {
        self.dim
    }

    pub fn set_dim(&mut self, dim: Vec2) {
        self.dim = dim;
    }
}

#[derive(Clone)]
pub struct TerminalCamera2dSettings {
    /// If enabled, rendering will attempt to stretch objects to fit the screen instead of rending each tile invdidually.
    stretch: bool,
    autoresize: bool,
}
impl Default for TerminalCamera2dSettings {
    fn default() -> Self {
        Self {
            stretch: false,
            autoresize: true,
        }
    }
}

impl TerminalCamera2dSettings {
    pub fn autoresize(&self) -> bool {
        self.autoresize
    }

    pub fn set_autoresize(&mut self, autoresize: bool) {
        self.autoresize = autoresize;
    }

    pub fn set_stretch(&mut self, stretch: bool) {
        self.stretch = stretch;
    }

    pub fn stretch(&self) -> bool {
        self.stretch
    }
}
