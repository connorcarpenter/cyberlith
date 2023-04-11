use bevy_ecs::world::World;

use render_egui::{egui, egui::Ui};

use crate::app::ui::{AxesCamerasVisible, UiState, WorkspaceType};

pub fn tab_bar(ui: &mut Ui, world: &mut World) {
    egui::menu::bar(ui, |ui| {
        let mut state = world.get_resource_mut::<UiState>().unwrap();
        if ui.add(egui::Button::new("Tab 1")).clicked() {
            state.workspace_type = WorkspaceType::SkeletonBuilder;
            let mut cameras_visible = world.get_resource_mut::<AxesCamerasVisible>().unwrap();
            (*cameras_visible).0 = true;
        } else if ui.add(egui::Button::new("Tab 2")).clicked() {
            state.workspace_type = WorkspaceType::TextEditor;
            let mut cameras_visible = world.get_resource_mut::<AxesCamerasVisible>().unwrap();
            (*cameras_visible).0 = false;
        } else if ui.add(egui::Button::new("Tab 3")).clicked() {
            state.workspace_type = WorkspaceType::None;
            let mut cameras_visible = world.get_resource_mut::<AxesCamerasVisible>().unwrap();
            (*cameras_visible).0 = false;
        }
    });
}
