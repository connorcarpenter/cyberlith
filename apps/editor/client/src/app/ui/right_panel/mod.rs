use bevy_ecs::world::World;

use render_egui::egui;

use crate::app::ui::UiState;

pub fn right_panel(
    context: &egui::Context,
    world: &mut World,
) {
    egui::SidePanel::right("right_panel")
        .resizable(false)
        .default_width(60.0)
        .show(context, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Tools");
            });

            ui.separator();

            ui.vertical_centered(|ui| {
                ui.label("T1");
            });
        });
}