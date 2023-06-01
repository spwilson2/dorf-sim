use crate::prelude::*;

use super::input::TerminalResize;

#[derive(Default)]
pub struct TerminalCamera2dPlugin();

#[derive(Default)]
pub struct CameraResized(pub UVec2);

impl Plugin for TerminalCamera2dPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerminalCamera2D::default())
            .add_startup_system(init_camera_autosize)
            .add_event::<CameraResized>()
            .add_system(handle_terminal_resize);
    }
}

fn init_camera_autosize(
    mut camera: ResMut<TerminalCamera2D>,
    mut camera_event_writer: EventWriter<CameraResized>,
) {
    if camera.settings().autoresize() {
        let term_size = crossterm::terminal::size().unwrap();
        let update = UVec2::new(term_size.0 as u32, term_size.1 as u32);
        camera.set_dim(update);
        camera_event_writer.send(CameraResized(update));
    }
}

fn handle_terminal_resize(
    mut camera: ResMut<TerminalCamera2D>,
    mut resize_reader: EventReader<TerminalResize>,
    mut camera_event_writer: EventWriter<CameraResized>,
) {
    if !camera.settings().autoresize() {
        return;
    }
    if let Some(resize) = resize_reader.iter().last() {
        let update = UVec2::new(resize.width as u32, resize.height as u32);
        if update != *camera.dim() {
            camera.set_dim(update);
            camera_event_writer.send(CameraResized(update));
        }
    }
}

#[derive(Resource, Default)]
pub struct TerminalCamera2D {
    pub transform: Transform2D,
    pub settings: TerminalCamera2DSettings,
}

impl TerminalCamera2D {
    pub fn new(loc: Vec3, scale: UVec2, z_lvl: i32) -> Self {
        Self {
            transform: Transform2D { scale, loc },
            settings: default(),
        }
    }
    pub fn transform(&self) -> &Transform2D {
        &self.transform
    }
    pub fn settings(&self) -> &TerminalCamera2DSettings {
        &self.settings
    }
    pub fn loc(&self) -> &Vec3 {
        &self.transform.loc
    }
    pub fn loc_mut(&mut self) -> &mut Vec3 {
        &mut self.transform.loc
    }
    pub fn dim(&self) -> &UVec2 {
        &self.transform.scale
    }
    pub fn set_dim(&mut self, dim: UVec2) {
        self.transform.scale = dim
    }
}

#[derive(Clone)]
pub struct TerminalCamera2DSettings {
    autoresize: bool,
}

impl Default for TerminalCamera2DSettings {
    fn default() -> Self {
        Self { autoresize: true }
    }
}

impl TerminalCamera2DSettings {
    pub fn autoresize(&self) -> bool {
        self.autoresize
    }

    pub fn set_autoresize(&mut self, autoresize: bool) {
        self.autoresize = autoresize;
    }
}
