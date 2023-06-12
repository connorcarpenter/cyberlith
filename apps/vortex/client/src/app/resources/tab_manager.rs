
use bevy_ecs::{
    prelude::{Entity, Resource},
    system::{Query, SystemState, ResMut},
    world::World,
};
use render_egui::{egui, egui::Ui};

use vortex_proto::components::FileSystemEntry;

use crate::app::{components::file_system::FileSystemUiState};

#[derive(Clone, Copy)]
pub struct TabState { pub selected: bool }

impl TabState {
    pub fn new(selected: bool) -> Self {
        Self { selected }
    }
}

#[derive(Resource)]
pub struct TabManager {
    tabs: Vec<(Entity, TabState)>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
        }
    }

    pub fn new_tab(&mut self, row_entity: &Entity) {
        self.tabs.push((*row_entity, TabState::new(true)));
    }

    pub fn deselect_all_tabs(&mut self) {
        for (_, tab_state) in self.tabs.iter_mut() {
            tab_state.selected = false;
        }
    }

    pub fn render_root(ui: &mut Ui, world: &mut World) {
        egui::menu::bar(ui, |ui| {
            let mut system_state: SystemState<(ResMut<TabManager>, Query<(&FileSystemEntry, &FileSystemUiState)>)> = SystemState::new(world);
            let (mut tab_manager, query) = system_state.get_mut(world);

            tab_manager.render_tabs(ui, &query);
        });
    }

    fn render_tabs(&mut self, ui: &mut Ui, query: &Query<(&FileSystemEntry, &FileSystemUiState)>) {

        let mut deselect_all = None;

        for i in 0..self.tabs.len() {

            let (entity, tab_state) = self.tabs.get_mut(i).unwrap();

            let (entry, ui_state) = query.get(*entity).unwrap();

            let text = &*entry.name;

            //TODO: put text on button in color from ui_state

            let mut button = egui::Button::new(text);
            if tab_state.selected {
                button = button.fill(egui::Color32::from_gray(113));
            }
            if ui.add(button).clicked() {
                deselect_all = Some(i);
            }
        }

        if let Some(i) = deselect_all {
            self.deselect_all_tabs();
            let (_, tab_state) = self.tabs.get_mut(i).unwrap();
            tab_state.selected = true;
        }
    }
}

// if ui.add(egui::Button::new("Tab 1")).clicked() {
//     let mut cameras_visible = world.get_resource_mut::<AxesCamerasVisible>().unwrap();
//     (*cameras_visible).0 = true;
//
// } else if ui.add(egui::Button::new("Tab 2")).clicked() {
//     let mut cameras_visible = world.get_resource_mut::<AxesCamerasVisible>().unwrap();
//     (*cameras_visible).0 = false;
//
// }