use bevy_ecs::{
    system::{Res, SystemState, Query, ResMut},
    world::World,
    entity::Entity,
};
use bevy_log::info;

use render_egui::{egui, egui::{Align, Layout, Ui}};
use vortex_proto::{components::FileSystemEntry, resources::FileEntryKey};

use crate::app::{resources::global::Global, ui::widgets::ChangelistRowUiWidget, components::file_system::FileSystemUiState};

#[derive(Clone, Copy)]
pub struct TabState { pub selected: bool }

impl TabState {
    pub fn new(selected: bool) -> Self {
        Self { selected }
    }
}

pub struct TabBarUiWidget;

impl TabBarUiWidget {
    pub fn render_root(ui: &mut Ui, world: &mut World) {
        egui::menu::bar(ui, |ui| {
            let mut system_state: SystemState<(ResMut<Global>, Query<(&FileSystemEntry, &FileSystemUiState)>)> = SystemState::new(world);
            let (mut global, query) = system_state.get_mut(world);

            global.render_tabs(ui, &query);

        });
    }
}

// let mut state = world.get_resource_mut::<UiState>().unwrap();
//
// if ui.add(egui::Button::new("Tab 1")).clicked() {
//
//     state.workspace_type = WorkspaceType::SkeletonBuilder;
//     let mut cameras_visible = world.get_resource_mut::<AxesCamerasVisible>().unwrap();
//     (*cameras_visible).0 = true;
//
// } else if ui.add(egui::Button::new("Tab 2")).clicked() {
//
//     state.workspace_type = WorkspaceType::TextEditor;
//     let mut cameras_visible = world.get_resource_mut::<AxesCamerasVisible>().unwrap();
//     (*cameras_visible).0 = false;
//
// } else if ui.add(egui::Button::new("Tab 3")).clicked() {
//
//     state.workspace_type = WorkspaceType::None;
//
//     let mut cameras_visible = world.get_resource_mut::<AxesCamerasVisible>().unwrap();
//     (*cameras_visible).0 = false;
// }