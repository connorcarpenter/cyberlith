use bevy_ecs::world::World;

use render_egui::egui;
use crate::app::plugin::ProjectTree;

use crate::app::ui::UiState;

pub fn left_panel(
    context: &egui::Context,
    world: &mut World,
) {
    egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(150.0)
        .show(context, |ui| {
            egui::TopBottomPanel::top("left_top_panel")
                .resizable(true)
                .default_height(300.0)
                .show_inside(ui, |ui| {
                    egui::TopBottomPanel::top("left_top_panel_header")
                        .show_inside(ui, |ui| {
                            ui.heading("Project");
                        });
                    egui::CentralPanel::default()
                        .show_inside(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
                                .show(ui, |ui| {
                                let mut tree = world.get_resource_mut::<ProjectTree>().unwrap();
                                tree.0.ui(ui);
                            });
                        });
                });
            egui::CentralPanel::default()
                .show_inside(ui, |ui| {
                    egui::TopBottomPanel::top("left_bottom_panel_header")
                        .show_inside(ui, |ui| {
                            ui.heading("Git Changes");
                        });
                    egui::CentralPanel::default()
                        .show_inside(ui, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.label("Changes File Tree");
                            });
                        });
                });

        });
}