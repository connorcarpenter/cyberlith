use bevy_ecs::{
    change_detection::DetectChanges,
    prelude::Mut,
    system::{Query, Res},
    world::World,
};
use bevy_log::info;
use render_api::components::Camera;
use render_egui::EguiContext;

use crate::app::{
    resources::action_stack::ActionStack,
    ui::{
        center_panel, left_panel, login_modal, right_panel, shortcuts::consume_shortcuts,
        text_input_modal::TextInputModal, top_bar, AxesCamerasVisible, UiState,
    },
};

pub fn main(world: &mut World) {
    let context = world.get_resource::<EguiContext>().unwrap().inner().clone();
    let ui_state = world.get_resource::<UiState>().unwrap();

    if ui_state.logged_in {
        top_bar(&context, world);
        left_panel(&context, world);
        right_panel(&context, world);
        center_panel(&context, world);
        TextInputModal::show(&context, world);

        consume_shortcuts(&context, world);

        world.resource_scope(|world, mut action_stack: Mut<ActionStack>| {
            action_stack.execute_actions(world);
        });
    } else {
        login_modal(&context, world);
    }
}

pub fn sync_axes_cameras_visibility(
    cameras_visible: Res<AxesCamerasVisible>,
    mut camera_q: Query<&mut Camera>,
) {
    if !cameras_visible.is_changed() {
        return;
    }

    let cameras_enabled = cameras_visible.0;

    if cameras_enabled {
        info!("Camera are ENABLED");
    } else {
        info!("Camera are DISABLED");
    }

    for mut camera in camera_q.iter_mut() {
        camera.is_active = cameras_enabled;
    }
}
