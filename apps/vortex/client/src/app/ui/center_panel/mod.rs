mod canvas;

use bevy_ecs::world::World;

use canvas::show_canvas;
use render_egui::{egui, egui::Frame};

use crate::app::{resources::tab_manager::TabManager, ui::right_panel};

pub fn center_panel(context: &egui::Context, world: &mut World) {
    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(0.0))
        .show(context, |ui| {
            egui::TopBottomPanel::top("tab_bar").show_inside(ui, |ui| {
                TabManager::render_root(ui, world);
            });
            right_panel(ui, world);
            egui::CentralPanel::default() // canvas area
                .frame(Frame::central_panel(ui.style()).inner_margin(0.0))
                .show_inside(ui, |ui| {
                    show_canvas(ui, world);
                });
        });
}
