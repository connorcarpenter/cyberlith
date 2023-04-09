use bevy_ecs::world::World;

use render_egui::{egui, egui::{Ui, Modifiers}};

use crate::app::ui::{UiState, WorkspaceType};

pub fn tab_bar(
    ui: &mut Ui,
    world: &mut World,
) {
    let mut state = world.get_resource_mut::<UiState>().unwrap();
    egui::menu::bar(ui, |ui| {
        if ui.add(egui::Button::new("Tab 1")).clicked() {
            state.workspace_type = WorkspaceType::SkeletonBuilder;
        }
        if ui.add(egui::Button::new("Tab 2")).clicked() {
            state.workspace_type = WorkspaceType::TextEditor;
        }
        if ui.add(egui::Button::new("Tab 3")).clicked() {
            state.workspace_type = WorkspaceType::None;
        }
    });
}