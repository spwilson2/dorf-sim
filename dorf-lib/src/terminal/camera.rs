use crate::prelude::*;

use super::input::TerminalResize;

#[derive(Default)]
pub struct TerminalCamera2dPlugin();

/// A tag component indicate the component should be place relative to the Camera.
#[derive(Component, Debug, Default)]
pub struct UIComponent {
    pub local_pos: Vec3,
    /// If true local_pos is in units [0, 1] and it is interpreted as a ratio of screen size
    /// If false, local_pos is an offset from topleft camera position.
    pub relative_pos: bool,
}

impl UIComponent {
    pub fn new(local_pos: Vec3) -> Self {
        Self {
            local_pos,
            relative_pos: false,
        }
    }
}

#[derive(Default)]
pub struct CameraResized(pub UVec2);

impl Plugin for TerminalCamera2dPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerminalCamera2D::default())
            .add_startup_system(init_camera_autosize)
            .add_event::<CameraResized>()
            .add_system(handle_terminal_resize)
            .add_system(sys_reposition_ui_elements_on_move.in_base_set(CoreSet::UpdateFlush));
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

fn sys_reposition_ui_elements_on_move(
    camera: Res<TerminalCamera2D>,
    mut ui_elems: Query<(&mut Transform2D, &UIComponent)>,
) {
    let update_ui_elem = |ui_transform: &mut Transform2D, ui_component: &UIComponent| {
        if ui_component.relative_pos {
            let z = ui_transform.loc.z;
            ui_transform.loc = (camera.transform.loc.xy()
                + camera.transform.scale.as_vec2() * ui_component.local_pos.xy())
            .min(camera.transform.as_rect2d().max.xy().as_vec2() - Vec2::ONE)
            .xyy();
            ui_transform.loc.z = z;
        } else {
            ui_transform.loc = ui_component.local_pos + camera.transform.loc;
        }
    };
    // TODO: Address inefficiency of the settings chaning affecting tihis. Might
    // be better if camera is a component bundle instead...

    //if !changed_ui_elems.is_empty() {
    //    // We need to at least update these elements to recenter them.
    //    for (mut transform, ui_component) in changed_ui_elems.iter_mut() {
    //        update_ui_elem(transform.as_mut(), ui_component)
    //    }
    //}

    if camera.is_changed() {
        // We need to recenter all elements..
        for (mut transform, ui_component) in ui_elems.iter_mut() {
            update_ui_elem(transform.as_mut(), ui_component)
        }
    } else {
        // XXX: Ideally, we could use a Query filter for this, but we can't
        // grab two mutable references at the same time. Not sure how to handle
        // other than by iterating over each one.
        for (mut transform, ui_component) in ui_elems.iter_mut() {
            if transform.is_changed() {
                update_ui_elem(transform.as_mut(), ui_component)
            }
        }
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

// TODO: Refactor as a Bundel, we don't these components bound together.. Ideally they wouldn't be.
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
