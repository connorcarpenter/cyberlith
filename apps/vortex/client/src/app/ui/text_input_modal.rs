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
use crate::app::ui::utils::ui_with_margin;

pub fn show_modal_rename(context: &egui::Context, world: &mut World) {

    let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
    if !ui_state.text_input_modal_open {
        return;
    }

    let modal = Modal::new(context, "rename_modal").with_close_on_outside_click(true);

    let was_open = modal.is_open();
    if !was_open {
        // Just opened
        modal.open();
    }

    let margin = 5.0;

    modal.show(|ui| {
        modal.title(ui, "Rename");
        modal.frame(ui, |ui| {

            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui_with_margin(ui, margin, |ui| {
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui_with_margin(ui, margin, |ui| {
                            ui.label("Rename file to:");
                        })
                    });
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        ui_with_margin(ui, margin, |ui| {
                            ui.text_edit_singleline(&mut ui_state.text_input_modal_value);
                        })
                    });
                })
            });
        });
        modal.buttons(ui, |ui| {
            if modal.button(ui, "Cancel").clicked() {
                info!("Cancel button Clicked!");
            }
            if modal.suggested_button(ui, "Rename").clicked() {
                info!("Rename button Clicked!");
            }
        });
    });

    if !modal.is_open() {
        // Just closed
        ui_state.text_input_modal_open = false;
    }
}