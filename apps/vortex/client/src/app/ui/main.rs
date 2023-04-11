use bevy_ecs::{
    change_detection::DetectChanges,
    event::Events,
    system::{Query, Res},
    world::World,
};
use bevy_log::info;
use egui_modal::{Modal, ModalStyle};
use render_api::components::Camera;
use render_egui::{
    egui,
    egui::{Align, Direction, Layout, Ui},
    EguiContext,
};

use crate::app::{
    events::LoginEvent,
    ui::{center_panel, left_panel, login_modal, right_panel, top_bar, UiState, WorkspaceType},
};

pub fn main(world: &mut World) {
    let context = world.get_resource::<EguiContext>().unwrap().inner().clone();
    let mut ui_state = world.get_resource_mut::<UiState>().unwrap();

    if ui_state.logged_in {
        top_bar(&context, world);
        left_panel(&context, world);
        right_panel(&context, world);
        center_panel(&context, world);
    } else {
        login_modal(&context, world);
    }
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
