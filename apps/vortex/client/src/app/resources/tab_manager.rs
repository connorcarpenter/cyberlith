use std::collections::HashMap;
use bevy_ecs::{
    prelude::{Entity, Resource},
    system::{Query, SystemState, ResMut},
    world::World,
};
use render_egui::{egui, egui::Ui};

use vortex_proto::components::FileSystemEntry;

use crate::app::{components::file_system::FileSystemUiState};

#[derive(Clone, Copy)]
pub struct TabState {
    pub selected: bool,
    pub order: usize,
}

impl TabState {
    pub fn new(order: usize) -> Self {
        Self {
            selected: false,
            order,
        }
    }
}

#[derive(Resource)]
pub struct TabManager {
    current_tab: Option<Entity>,
    tab_map: HashMap<Entity, TabState>,
    tab_order: Vec<Entity>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            current_tab: None,
            tab_map: HashMap::new(),
            tab_order: Vec::new(),
        }
    }

    pub fn open_tab(&mut self, row_entity: &Entity) {

        if self.tab_map.contains_key(row_entity) {
            self.select_current_tab(row_entity);
        } else {

            // get current tab order
            let current_order = if let Some(current_entity) = self.current_tab {
                let tab_state = self.tab_map.get(&current_entity).unwrap();
                tab_state.order + 1
            } else {
                0
            };

            // insert new tab
            self.tab_map.insert(*row_entity, TabState::new(current_order));
            self.tab_order.insert(current_order, *row_entity);
            self.select_current_tab(row_entity);

            // update tab orders
            for (i, entity) in self.tab_order.iter_mut().enumerate() {
                let tab_state = self.tab_map.get_mut(entity).unwrap();
                tab_state.order = i;
            }
        }
    }

    fn select_current_tab(&mut self, row_entity: &Entity) {

        // deselect current tab
        if let Some(current_entity) = self.current_tab {
            let tab_state = self.tab_map.get_mut(&current_entity).unwrap();
            tab_state.selected = false;
        }

        // select new tab
        self.current_tab = Some(*row_entity);
        let tab_state = self.tab_map.get_mut(&row_entity).unwrap();
        tab_state.selected = true;
    }

    pub fn render_root(ui: &mut Ui, world: &mut World) {
        egui::menu::bar(ui, |ui| {
            let mut system_state: SystemState<(ResMut<TabManager>, Query<(&FileSystemEntry, &FileSystemUiState)>)> = SystemState::new(world);
            let (mut tab_manager, query) = system_state.get_mut(world);

            tab_manager.render_tabs(ui, &query);
        });
    }

    fn render_tabs(&mut self, ui: &mut Ui, query: &Query<(&FileSystemEntry, &FileSystemUiState)>) {

        let mut clicked_tab = None;

        for row_entity in &self.tab_order {

            let tab_state = self.tab_map.get_mut(row_entity).unwrap();

            let (entry, ui_state) = query.get(*row_entity).unwrap();

            let text = format!("📃 {}", &*entry.name);

            //TODO: put text on button in color from ui_state

            let mut button = egui::Button::new(text);
            if tab_state.selected {
                button = button.fill(egui::Color32::from_gray(113));
            }
            let button_response = ui.add(button);
            if button_response.clicked() {
                clicked_tab = Some(*row_entity);
            }

            // Tab context menu
            button_response.context_menu(|ui| {
                if ui
                    .add(egui::Button::new("Close"))
                    .clicked()
                {
                    // TODO
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Close Other Tabs"))
                    .clicked()
                {
                    // TODO
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Close All Tabs"))
                    .clicked()
                {
                    // TODO
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Close Tabs to the Left"))
                    .clicked()
                {
                    // TODO
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Close Tabs to the Right"))
                    .clicked()
                {
                    // TODO
                    ui.close_menu();
                }
            });

        }

        if let Some(row_entity) = clicked_tab {
            self.select_current_tab(&row_entity);
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