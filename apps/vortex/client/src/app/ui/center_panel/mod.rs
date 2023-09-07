mod canvas;

use bevy_ecs::world::World;

use canvas::render_canvas;
use render_egui::{egui, egui::Frame};

use crate::app::{ui::{UiState, render_tool_bar, widgets::render_naming_bar}, resources::tab_manager::render_tab_bar};

pub fn center_panel(context: &egui::Context, world: &mut World) {
    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(0.0))
        .show(context, |ui| {
            render_tab_bar(ui, world);
            render_tool_bar(ui, world);

            let ui_state = world.get_resource::<UiState>().unwrap();
            if ui_state.naming_bar_visible {
                egui::CentralPanel::default() // canvas area
                    .frame(Frame::central_panel(ui.style()).inner_margin(0.0))
                    .show_inside(ui, |ui| {
                        render_naming_bar(ui, world);
                        render_canvas(ui, world);
                    });
            } else {
                render_canvas(ui, world);
            }
        });
}
