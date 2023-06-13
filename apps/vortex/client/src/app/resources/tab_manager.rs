use std::collections::HashMap;
use bevy_ecs::{
    prelude::{Entity, Resource},
    system::{Query, SystemState, ResMut},
    world::World,
};
use render_egui::{egui, egui::{Ui, RichText, WidgetText, Id, Label, NumExt, Rect, Response, Rounding, Sense, Stroke, TextStyle, vec2}};

use vortex_proto::components::{ChangelistStatus, FileSystemEntry};

use crate::app::{components::file_system::FileSystemUiState, ui::widgets::colors::{TEXT_COLORS_SELECTED, FILE_ROW_COLORS_HOVER, FILE_ROW_COLORS_SELECTED, FILE_ROW_COLORS_UNSELECTED, TEXT_COLORS_HOVER, TEXT_COLORS_UNSELECTED}};


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

            let tab_state = self.tab_map.get(row_entity).unwrap();

            let (entry, ui_state) = query.get(*row_entity).unwrap();

            //let button_response = Self::old_tab_render(ui, row_entity, entry, ui_state, tab_state, &mut tab_action);
            let button_response = Self::new_tab_render(ui, row_entity, entry, ui_state, tab_state, &mut tab_action);

            Self::tab_context_menu(button_response, row_entity, &mut tab_action);
        }

        self.execute_tab_action(tab_action);
    }

    fn old_tab_render(
        ui: &mut Ui,
        row_entity: &Entity,
        entry: &FileSystemEntry,
        ui_state: &FileSystemUiState,
        tab_state: &TabState,
        tab_action: &mut Option<TabAction>
    ) -> Response {
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
            *tab_action = Some(TabAction::Select(*row_entity));
        }

        button_response
    }

    fn new_tab_render(
        ui: &mut Ui,
        row_entity: &Entity,
        entry: &FileSystemEntry,
        ui_state: &FileSystemUiState,
        tab_state: &TabState,
        tab_action: &mut Option<TabAction>
    ) -> Response {
        let x_icon_nohover = "❌";
        let x_icon_hover = "❎";
        let file_name = &*entry.name;
        let full_path = format!("tab_cancel_button:{:?}", row_entity);
        let file_name_str = format!("📃 {}", file_name);

        let file_name_galley = WidgetText::from(file_name_str).into_galley(ui, Some(false), 1.0, TextStyle::Button);

        let file_name_text_size = file_name_galley.size();
        let mut desired_tab_size = file_name_text_size;
        desired_tab_size.x += 22.0; // make room for close button
        desired_tab_size.y = desired_tab_size.y.at_least(ui.spacing().interact_size.y);

        let (mut tab_rect, tab_response) = ui.allocate_at_least(desired_tab_size, Sense::click());

        if ui.is_rect_visible(tab_response.rect) {
            let item_spacing = 5.0;

            let mut text_position = tab_rect.min;
            text_position.y += 2.0;
            let mut icon_position = text_position;
            icon_position.x += file_name_text_size.x + item_spacing;

            // adjust tab rect size
            tab_rect.min.x -= 8.0;
            tab_rect.min.y -= 2.0;
            tab_rect.max.y += 2.0;

            let icon_response = {
                let icon_size = vec2(ui.spacing().icon_width, ui.spacing().icon_width);
                let icon_rect = Rect::from_min_size(icon_position, icon_size);

                let big_icon_response = ui.interact(icon_rect, Id::new(full_path), Sense::click());

                if big_icon_response.clicked() {
                    *tab_action = Some(TabAction::Close(*row_entity));
                } else {
                    if tab_response.clicked() {
                        *tab_action = Some(TabAction::Select(*row_entity));
                    }
                }

                big_icon_response
            };

            let (text_colors, row_fill_colors) = {
                if tab_state.selected {
                    (TEXT_COLORS_SELECTED, FILE_ROW_COLORS_SELECTED)
                } else {
                    if tab_response.hovered() || icon_response.hovered() {
                        (TEXT_COLORS_HOVER, FILE_ROW_COLORS_HOVER)
                    } else {
                        (TEXT_COLORS_UNSELECTED, FILE_ROW_COLORS_UNSELECTED)
                    }
                }
            };

            // Draw Row
            {
                let row_fill_color_opt = row_fill_colors.available;

                if let Some(row_fill_color) = row_fill_color_opt {
                    ui.painter()
                        .rect(tab_rect, Rounding::none(), row_fill_color, Stroke::NONE);
                }
            }

            // Draw Text
            {
                let text_color = match ui_state.change_status {
                    Some(ChangelistStatus::Created) => text_colors.created,
                    Some(ChangelistStatus::Modified) => text_colors.modified,
                    _ => text_colors.default,
                };
                file_name_galley.paint_with_color_override(ui.painter(), text_position, text_color);
            }

            // Draw Icon
            {
                let (small_icon_rect, _) = ui.spacing().icon_rectangles(icon_response.rect);
                let small_icon_response = icon_response.clone().with_new_rect(small_icon_rect);
                let x_icon_text = match small_icon_response.hovered() {
                    true => x_icon_hover,
                    false => x_icon_nohover,
                };

                let x_icon_galley = WidgetText::from(x_icon_text).into_galley(ui, Some(false), 1.0, TextStyle::Button);
                x_icon_galley.paint_with_color_override(ui.painter(), icon_position, text_colors.default);
            }
        }

        tab_response
    }

    fn tab_context_menu(button_response: Response, row_entity: &Entity, tab_action: &mut Option<TabAction>) {
        // Tab context menu
        button_response.context_menu(|ui| {
            if ui
                .add(egui::Button::new("Close"))
                .clicked()
            {
                *tab_action = Some(TabAction::Close(*row_entity));
                ui.close_menu();
            }
            if ui
                .add(egui::Button::new("Close Other Tabs"))
                .clicked()
            {
                *tab_action = Some(TabAction::CloseOthers(*row_entity));
                ui.close_menu();
            }
            if ui
                .add(egui::Button::new("Close All Tabs"))
                .clicked()
            {
                *tab_action = Some(TabAction::CloseAll);
                ui.close_menu();
            }
            if ui
                .add(egui::Button::new("Close Tabs to the Left"))
                .clicked()
            {
                *tab_action = Some(TabAction::CloseLeft(*row_entity));
                ui.close_menu();
            }
            if ui
                .add(egui::Button::new("Close Tabs to the Right"))
                .clicked()
            {
                *tab_action = Some(TabAction::CloseRight(*row_entity));
                ui.close_menu();
            }
        });
    }

    fn execute_tab_action(&mut self, tab_action: Option<TabAction>) {
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