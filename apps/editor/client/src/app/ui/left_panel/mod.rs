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
            // Left Bottom Panel
            egui::TopBottomPanel::bottom("left_bottom_panel")
                .resizable(true)
                .show_inside(ui, |ui| {
                    ui.heading("Git Changes");
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .always_show_scroll(true)
                        .show(ui, |ui| {
                            let mut tree = world.get_resource_mut::<ProjectTree>().unwrap();
                            tree.0.ui(ui);
                            ui.allocate_space(ui.available_size());
                        });
                });
            // Left Top Panel
            egui::CentralPanel::default()
                .show_inside(ui, |ui| {
                    ui.heading("Project");
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .auto_shrink([true, true])
                        .always_show_scroll(true)
                        .show(ui, |ui| {
                            let mut tree = world.get_resource_mut::<ProjectTree>().unwrap();
                            tree.0.ui(ui);
                        });
                });

        });
}