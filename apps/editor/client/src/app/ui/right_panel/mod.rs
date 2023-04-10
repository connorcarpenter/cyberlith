use bevy_ecs::world::World;

use render_egui::{egui, egui::Frame};

use crate::app::ui::UiState;

pub fn right_panel(context: &egui::Context, world: &mut World) {
    egui::SidePanel::right("right_panel")
        .frame(Frame::none().inner_margin(0.0))
        .resizable(false)
        .default_width(60.0)
        .show(context, |ui| {
            egui::TopBottomPanel::top("right_panel_header").show_inside(ui, |ui| {
                ui.heading("Tools");
            });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.add(egui::Button::new("tool 1"));
            });
        });
}
