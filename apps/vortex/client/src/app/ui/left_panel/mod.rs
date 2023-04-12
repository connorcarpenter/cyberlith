use bevy_ecs::world::World;

use render_egui::{egui, egui::Frame};

use crate::app::plugin::{ChangesTree, ProjectTree};

pub fn left_panel(context: &egui::Context, world: &mut World) {
    egui::SidePanel::left("left_panel")
        .frame(Frame::none().inner_margin(0.0))
        .resizable(true)
        .default_width(150.0)
        .show(context, |ui| {
            // Left Top Panel
            egui::TopBottomPanel::top("left_top_panel")
                .frame(Frame::side_top_panel(ui.style()).inner_margin(0.0))
                .resizable(true)
                .default_height(ui.available_height() * 0.5)
                .show_inside(ui, |ui| {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.heading("Project");
                    });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let mut tree = world.get_resource_mut::<ProjectTree>().unwrap();
                            tree.0.render_root(ui);
                        });
                });
            // Left Bottom Panel
            egui::CentralPanel::default()
                .frame(Frame::side_top_panel(ui.style()).inner_margin(0.0))
                .show_inside(ui, |ui| {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.heading("Git Changes");
                    });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let mut tree = world.get_resource_mut::<ChangesTree>().unwrap();
                            tree.0.render_root(ui);
                            ui.allocate_space(ui.available_size());
                        });
                });
        });
}
