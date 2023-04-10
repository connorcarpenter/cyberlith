use std::borrow::BorrowMut;

use bevy_ecs::{
    change_detection::DetectChanges,
    system::{Query, Res, ResMut},
    world::World,
};
use bevy_log::info;

use render_api::components::Camera;
use render_egui::{egui, EguiContext};

use crate::app::ui::{center_panel, left_panel, right_panel, top_bar, UiState, WorkspaceType};

pub fn main(world: &mut World) {
    let context = world.get_resource::<EguiContext>().unwrap().inner().clone();
    top_bar(&context, world);
    left_panel(&context, world);
    right_panel(&context, world);
    center_panel(&context, world);
}

pub fn sync_ui_to_world(ui_state: Res<UiState>, mut camera_q: Query<&mut Camera>) {
    if !ui_state.is_changed() {
        return;
    }

    let cameras_enabled = ui_state.workspace_type == WorkspaceType::SkeletonBuilder;

    if cameras_enabled {
        info!("Camera are ENABLED");
    } else {
        info!("Camera are DISABLED");
    }

    for mut camera in camera_q.iter_mut() {
        camera.is_active = cameras_enabled;
    }
}
