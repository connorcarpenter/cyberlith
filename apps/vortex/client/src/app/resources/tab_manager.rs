use std::collections::HashMap;
use bevy_ecs::{
    prelude::{Entity, Resource},
    system::{Query, SystemState, ResMut},
    world::World,
};
use render_egui::{egui, egui::{Ui, RichText, WidgetText}};

use vortex_proto::components::{ChangelistStatus, FileSystemEntry};

use crate::app::{components::file_system::FileSystemUiState, ui::widgets::colors::TEXT_COLORS_SELECTED};
use crate::app::ui::widgets::colors::TEXT_COLORS_UNSELECTED;

#[derive(Clone, Copy)]
struct TabState {
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

enum TabAction {
    Select(Entity),
    Close(Entity),
    CloseAll,
    CloseOthers(Entity),
    CloseLeft(Entity),
    CloseRight(Entity),
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
            self.select_tab(row_entity);
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
            self.select_tab(row_entity);

            self.update_tab_orders();
        }
    }

    fn update_tab_orders(&mut self) {
        for (i, entity) in self.tab_order.iter_mut().enumerate() {
            let tab_state = self.tab_map.get_mut(entity).unwrap();
            tab_state.order = i;
        }
    }

    fn select_tab(&mut self, row_entity: &Entity) {

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

    fn close_tab(&mut self, row_entity: &Entity) {

        // remove tab
        let tab_state = self.tab_map.remove(row_entity).unwrap();
        self.tab_order.remove(tab_state.order);

        self.update_tab_orders();

        // select new tab
        if let Some(current_entity) = self.current_tab {
            if current_entity == *row_entity {
                let mut new_tab_order = tab_state.order;
                if new_tab_order > 0 {
                    new_tab_order -= 1;
                }
                if let Some(new_entity) = self.tab_order.get(new_tab_order) {
                    let new_entity = *new_entity;
                    self.current_tab = None;
                    self.select_tab(&new_entity);
                } else {
                    self.current_tab = None;
                }
            }
        }
    }

    fn close_all_tabs(&mut self) {
        self.tab_map.clear();
        self.tab_order.clear();
        self.current_tab = None;
    }

    fn close_all_tabs_except(&mut self, row_entity: &Entity) {
        self.close_all_tabs();
        self.open_tab(row_entity);
    }

    fn close_all_tabs_left_of(&mut self, row_entity: &Entity) {
        let tab_state = self.tab_map.get(row_entity).unwrap();
        let order = tab_state.order;
        let mut tabs_to_close: Vec<Entity> = Vec::new();
        for i in 0..order {
            let entity = self.tab_order[i];
            tabs_to_close.push(entity);
        }

        for entity in tabs_to_close {
            self.close_tab(&entity);
        }
    }

    fn close_all_tabs_right_of(&mut self, row_entity: &Entity) {
        let tab_state = self.tab_map.get(row_entity).unwrap();
        let order = tab_state.order;
        let mut tabs_to_close: Vec<Entity> = Vec::new();
        for i in order+1..self.tab_order.len() {
            let entity = self.tab_order[i];
            tabs_to_close.push(entity);
        }

        for entity in tabs_to_close {
            self.close_tab(&entity);
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

        let mut tab_action = None;

        for row_entity in &self.tab_order {

            let tab_state = self.tab_map.get_mut(row_entity).unwrap();

            let (entry, ui_state) = query.get(*row_entity).unwrap();

            let mut text: RichText = format!("📃 {}", &*entry.name).into();
            text = match ui_state.change_status {
                Some(ChangelistStatus::Modified) => {
                    text.color(TEXT_COLORS_UNSELECTED.modified)
                }
                Some(ChangelistStatus::Created) => {
                    text.color(TEXT_COLORS_UNSELECTED.created)
                }
                _ => {
                    text
                }
            };

            let mut button = egui::Button::new(WidgetText::RichText(text));
            if tab_state.selected {
                button = button.fill(egui::Color32::from_gray(113));
            }
            let button_response = ui.add(button);
            if button_response.clicked() {
                tab_action = Some(TabAction::Select(*row_entity));
            }

            // Tab context menu
            button_response.context_menu(|ui| {
                if ui
                    .add(egui::Button::new("Close"))
                    .clicked()
                {
                    tab_action = Some(TabAction::Close(*row_entity));
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Close Other Tabs"))
                    .clicked()
                {
                    tab_action = Some(TabAction::CloseOthers(*row_entity));
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Close All Tabs"))
                    .clicked()
                {
                    tab_action = Some(TabAction::CloseAll);
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Close Tabs to the Left"))
                    .clicked()
                {
                    tab_action = Some(TabAction::CloseLeft(*row_entity));
                    ui.close_menu();
                }
                if ui
                    .add(egui::Button::new("Close Tabs to the Right"))
                    .clicked()
                {
                    tab_action = Some(TabAction::CloseRight(*row_entity));
                    ui.close_menu();
                }
            });
        }

        match tab_action {
            None => {}
            Some(TabAction::Select(row_entity)) => {
                self.select_tab(&row_entity);
            }
            Some(TabAction::Close(row_entity)) => {
                self.close_tab(&row_entity);
            }
            Some(TabAction::CloseAll) => {
                self.close_all_tabs();
            }
            Some(TabAction::CloseOthers(row_entity)) => {
                self.close_all_tabs_except(&row_entity);
            }
            Some(TabAction::CloseLeft(row_entity)) => {
                self.close_all_tabs_left_of(&row_entity);
            }
            Some(TabAction::CloseRight(row_entity)) => {
                self.close_all_tabs_right_of(&row_entity);
            }
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