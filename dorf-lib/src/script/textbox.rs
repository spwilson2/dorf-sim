use crate::prelude::*;

#[derive(Component, Default, Clone, Debug)]
pub struct TextboxOptions {
    local_transfrm: Transform2D,
    framed: bool,
    autosize: bool,
}

#[derive(Bundle, Debug, Clone)]
pub struct Textbox {
    pub mesh: CharMeshTransform,
    tag: TextboxOptions,
}

impl Textbox {
    // TODO: Need ability to keep frame autosized?
    // TODO: Need ability to animate text/update it
    pub fn new() -> Self {
        Self {
            tag: default(),
            mesh: todo!(),
        }
    }

    pub fn resize(&mut self, size: UVec2) {
        todo!()
    }

    fn sys_resize_frame() {
        todo!()
    }

    fn sys_keep_relative_camera(
        camera: &mut ResMut<TerminalCamera2D>,
        self_: Query<(&CharMesh, &TextboxOptions)>,
    ) {
        // Automatically update our transform based on the camera's transform.
        // TODO: THis is going to be a general problem, we should add some
        // component tag or something to make this implementation free.
        todo!()
    }
}
