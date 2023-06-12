use std::collections::BTreeMap;

use bevy_ecs::{prelude::{Entity, Resource}, system::Query};
use render_egui::{egui, egui::Ui};

use vortex_proto::{components::FileSystemEntry, resources::FileEntryKey};

use crate::app::{components::file_system::FileSystemUiState, ui::widgets::TabState};

#[derive(Resource)]
pub struct Global {
    pub project_root_entity: Entity,
    pub changelist: BTreeMap<FileEntryKey, Entity>,
    pub tabs: Vec<(Entity, TabState)>,
}

impl Global {
    pub fn new(project_root_entity: Entity) -> Self {
        Self {
            project_root_entity,
            changelist: BTreeMap::new(),
            tabs: Vec::new(),
        }
    }

    pub fn deselect_all_tabs(&mut self) {
        for (_, tab_state) in self.tabs.iter_mut() {
            tab_state.selected = false;
        }
    }

    pub fn render_tabs(&mut self, ui: &mut Ui, query: &Query<(&FileSystemEntry, &FileSystemUiState)>) {

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