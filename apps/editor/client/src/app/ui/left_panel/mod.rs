use bevy_ecs::world::World;

use render_egui::egui;

use crate::app::ui::UiState;

pub fn left_panel(
    context: &egui::Context,
    world: &mut World,
) {
    egui::SidePanel::left("left_panel")
        .resizable(true)
        .default_width(150.0)
        .show(context, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Project");
            });

            ui.separator();

            ui.vertical_centered(|ui| {
                ui.label("file tree here");
            });
        });
}