use bevy_ecs::{
    change_detection::DetectChanges,
    system::{Query, Res},
    world::World,
};
use bevy_log::info;
use egui_modal::{Modal, ModalStyle};
use render_api::components::Camera;
use render_egui::{egui, EguiContext, egui::Layout};
use render_egui::egui::{Align, Direction, Ui};

use crate::app::ui::{center_panel, left_panel, right_panel, top_bar, UiState, WorkspaceType};

pub fn main(world: &mut World) {
    let context = world.get_resource::<EguiContext>().unwrap().inner().clone();
    let mut ui_state = world.get_resource_mut::<UiState>().unwrap();

    if ui_state.logged_in {
        top_bar(&context, world);
        left_panel(&context, world);
        right_panel(&context, world);
        center_panel(&context, world);
    } else {
        login_modal(&context, &mut ui_state);
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

fn login_modal(context: &egui::Context, ui_state: &mut UiState) {
    let modal = Modal::new(context, "login_modal");
    let margin = 5.0;
    modal.show(|ui| {
        modal.frame(ui, |ui| {

            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui_with_margin(ui, margin, |ui| {
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui_with_margin(ui, margin, |ui| {
                            ui.label("username: ");
                            ui.text_edit_singleline(&mut ui_state.username);
                        })
                    });
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui_with_margin(ui, margin, |ui| {
                            ui.label("password: ");
                            ui.text_edit_singleline(&mut ui_state.password);
                        })
                    });
                })
            });
        });
        ui.separator();
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            modal.button(ui, "login");
        });
    });
    modal.open();
}

fn ui_with_margin<R>(ui: &mut Ui, margin: f32, add_contents: impl FnOnce(&mut Ui) -> R) {
    egui::Frame::none()
        .inner_margin(margin)
        .show(ui, |ui| add_contents(ui));
}
