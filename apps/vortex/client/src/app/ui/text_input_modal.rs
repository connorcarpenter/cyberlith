use bevy_ecs::{event::Events, world::World};
use bevy_log::info;

use egui_modal::Modal;

use render_egui::{
    egui,
    egui::{Align, Layout, Ui},
};

use crate::app::{
    config::AppConfig,
    events::LoginEvent,
    ui::{LoggingInState, UiState},
};

pub fn show_modal_rename(context: &egui::Context, world: &mut World) {

    let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
    if !ui_state.text_input_modal_open {
        return;
    }

    let modal = Modal::new(context, "rename_modal");

    modal.show(|ui| {
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.label("rename file to");
            ui.text_edit_singleline(&mut ui_state.text_input_modal_value)
        });

        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            if modal.button(ui, "rename").clicked() {
                info!("Rename button Clicked!");
            }
            if modal.button(ui, "cancel").clicked() {
                info!("Cancel button Clicked!");
            }
        });
    });

    modal.open();
}

fn ui_with_margin<R>(ui: &mut Ui, margin: f32, add_contents: impl FnOnce(&mut Ui) -> R) {
    egui::Frame::none()
        .inner_margin(margin)
        .show(ui, |ui| add_contents(ui));
}