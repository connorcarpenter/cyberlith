use bevy_ecs::{
    system::{NonSendMut, Res, ResMut},
    world::World,
};

use render_glow::window::FrameInput;

use crate::{EguiContext, GUI};

pub fn startup(world: &mut World) {
    world.insert_non_send_resource(GUI::default());
}

pub fn pre_update(
    mut gui: NonSendMut<GUI>,
    egui_context: Res<EguiContext>,
    mut frame_input: NonSendMut<FrameInput<()>>,
) {
    gui.pre_update(egui_context.inner(), frame_input.as_mut());
}

pub fn post_update(
    mut gui: NonSendMut<GUI>,
    egui_context: Res<EguiContext>,
    mut frame_input: NonSendMut<FrameInput<()>>,
) {
    gui.post_update(egui_context.inner(), &mut frame_input.events);
}

pub fn draw(mut gui: NonSendMut<GUI>, egui_context: Res<EguiContext>) {
    gui.render(egui_context.inner());
}
