use std::collections::{HashMap, VecDeque};

use bevy_ecs::{
    prelude::{Entity, Resource},
    system::{Query, Res, ResMut, SystemState},
    world::{Mut, World},
};

use naia_bevy_client::Client;

use render_api::components::Visibility;
use render_egui::{
    egui,
    egui::{vec2, Id, NumExt, Rect, Response, Rounding, Sense, Stroke, TextStyle, Ui, WidgetText},
};

use vortex_proto::{
    channels::TabActionChannel,
    components::{ChangelistStatus, FileExtension, FileSystemEntry},
    messages::{TabCloseMessage, TabOpenMessage},
    types::TabId,
};

use crate::app::{
    components::{file_system::FileSystemUiState, OwnedByFileLocal},
    resources::{
        action::AnimAction,
        action::{ShapeAction, TabActionStack},
        animation_manager::AnimationManager,
        camera_manager::CameraManager,
        camera_state::CameraState,
        canvas::Canvas,
        edge_manager::EdgeManager,
        file_manager::FileManager,
        input_manager::InputManager,
        shape_data::CanvasShape,
        shape_manager::ShapeManager,
        vertex_manager::VertexManager,
    },
    ui::widgets::colors::{
        FILE_ROW_COLORS_HOVER, FILE_ROW_COLORS_SELECTED, FILE_ROW_COLORS_UNSELECTED,
        TEXT_COLORS_HOVER, TEXT_COLORS_SELECTED, TEXT_COLORS_UNSELECTED,
    },
};

pub struct TabState {
    pub selected: bool,
    pub order: usize,
    pub tab_id: TabId,
    pub camera_state: CameraState,
    pub action_stack: TabActionStack,
    selected_shape_2d: Option<Option<(Entity, CanvasShape)>>,
}

impl TabState {
    pub fn new(id: TabId, order: usize, file_ext: FileExtension) -> Self {
        Self {
            selected: false,
            order,
            tab_id: id,
            camera_state: CameraState::default(),
            action_stack: TabActionStack::new(file_ext),
            selected_shape_2d: None,
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
    last_tab: Option<Entity>,
    resync_tab_ownership: bool,
    // entity is file entity here
    tab_map: HashMap<Entity, TabState>,
    tab_order: Vec<Entity>,
    new_tab_id: TabId,
    recycled_tab_ids: VecDeque<TabId>,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
            current_tab: None,
            last_tab: None,
            resync_tab_ownership: false,
            tab_map: HashMap::new(),
            tab_order: Vec::new(),
            new_tab_id: 0,
            recycled_tab_ids: VecDeque::new(),
        }
    }
}

impl TabManager {
    pub fn sync_tabs(world: &mut World) {
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.on_sync_tabs(world);
            tab_manager.on_sync_tab_ownership(world);
        });
    }

    pub fn on_sync_tabs(&mut self, world: &mut World) {
        if self.current_tab == self.last_tab {
            return;
        }

        if let Some(last_tab_entity) = self.last_tab {
            let last_selected_shape = world
                .get_resource::<InputManager>()
                .unwrap()
                .selected_shape_2d();
            if let Some(tab_state) = self.tab_map.get_mut(&last_tab_entity) {
                tab_state.selected_shape_2d = Some(last_selected_shape);
            }
        }

        self.last_tab = self.current_tab;

        let mut system_state: SystemState<(
            ResMut<Canvas>,
            Res<FileManager>,
            ResMut<InputManager>,
            ResMut<CameraManager>,
            ResMut<VertexManager>,
            ResMut<EdgeManager>,
            ResMut<AnimationManager>,
        )> = SystemState::new(world);
        let (
            mut canvas,
            file_manager,
            mut input_manager,
            mut camera_manager,
            mut vertex_manager,
            mut edge_manager,
            mut animation_manager,
        ) = system_state.get_mut(world);

        let mut canvas_is_visible = false;
        if let Some(current_file_entity) = self.current_tab {
            let current_file_type = file_manager.get_file_type(&current_file_entity);
            canvas_is_visible = match current_file_type {
                FileExtension::Palette => false,
                FileExtension::Skel | FileExtension::Mesh | FileExtension::Anim => true,
                _ => false,
            };
        }

        if canvas_is_visible {
            canvas.set_visibility(true);
            canvas.set_focused_timed(
                &mut input_manager,
                &mut vertex_manager,
                &mut edge_manager,
                &mut animation_manager,
            );
        } else {
            canvas.set_visibility(false);
        }

        input_manager.deselect_shape(&mut canvas);
        if let Some(tab_state) = self.current_tab_state_mut() {
            if let Some(Some((entity, shape))) = tab_state.selected_shape_2d.take() {
                input_manager.select_shape(&mut canvas, &entity, shape);
            }
        }

        camera_manager.recalculate_3d_view();
        self.resync_tab_ownership();
    }

    pub fn resync_tab_ownership(&mut self) {
        self.resync_tab_ownership = true;
    }

    pub fn on_sync_tab_ownership(&mut self, world: &mut World) {
        if !self.resync_tab_ownership {
            return;
        }

        self.resync_tab_ownership = false;

        let mut system_state: SystemState<(
            Res<FileManager>,
            ResMut<Canvas>,
            Query<(&mut Visibility, &OwnedByFileLocal)>,
        )> = SystemState::new(world);
        let (file_manager, mut canvas, mut visibility_q) = system_state.get_mut(world);

        if let Some(current_tab_file_entity) = self.current_tab_entity() {
            for (mut visibility, owned_by_tab) in visibility_q.iter_mut() {
                visibility.visible = ShapeManager::is_owned_by_file(
                    &file_manager,
                    current_tab_file_entity,
                    Some(&owned_by_tab.file_entity),
                );
            }
        }

        canvas.queue_resync_shapes();
    }

    pub fn open_tab(&mut self, client: &mut Client, row_entity: &Entity, file_ext: FileExtension) {
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
            let new_tab_id = self.new_tab_id();
            self.tab_map.insert(
                *row_entity,
                TabState::new(new_tab_id, current_order, file_ext),
            );
            self.tab_order.insert(current_order, *row_entity);

            // send message to server
            let message = TabOpenMessage::new(client, new_tab_id, row_entity);
            client.send_message::<TabActionChannel, TabOpenMessage>(&message);

            // select tab
            self.select_tab(row_entity);

            self.update_tab_orders();
        }
    }

    pub fn close_tab(&mut self, client: &mut Client, row_entity: &Entity) {
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
                    self.clear_current_tab();
                    self.select_tab(&new_entity);
                } else {
                    self.clear_current_tab();
                }
            }
        }

        // send message to server
        let message = TabCloseMessage::new(tab_state.tab_id);
        client.send_message::<TabActionChannel, TabCloseMessage>(&message);

        // recycle tab id
        self.recycle_tab_id(tab_state.tab_id);
    }

    pub fn file_has_tab(&self, file_entity: &Entity) -> bool {
        self.tab_map.contains_key(file_entity)
    }

    pub fn tab_state_mut(&mut self, file_entity: &Entity) -> Option<&mut TabState> {
        self.tab_map.get_mut(file_entity)
    }

    pub fn current_tab_entity(&self) -> Option<&Entity> {
        self.current_tab.as_ref()
    }

    pub fn current_tab_state(&self) -> Option<&TabState> {
        let current_entity = self.current_tab?;
        let tab_state = self.tab_map.get(&current_entity)?;
        Some(tab_state)
    }

    pub fn current_tab_state_mut(&mut self) -> Option<&mut TabState> {
        let current_entity = self.current_tab?;
        let tab_state = self.tab_map.get_mut(&current_entity)?;
        Some(tab_state)
    }

    pub fn current_tab_execute_shape_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        action: ShapeAction,
    ) {
        let current_tab_entity = *self.current_tab_entity().unwrap();
        let tab_state = self.current_tab_state_mut().unwrap();
        tab_state.action_stack.execute_shape_action(
            world,
            input_manager,
            current_tab_entity,
            action,
        );
    }

    pub fn current_tab_execute_anim_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        action: AnimAction,
    ) {
        let tab_file_entity = *self.current_tab_entity().unwrap();
        let tab_state = self.current_tab_state_mut().unwrap();
        tab_state
            .action_stack
            .execute_anim_action(world, input_manager, tab_file_entity, action);
    }

    pub fn current_tab_camera_state_mut(&mut self) -> Option<&mut CameraState> {
        let current_entity = self.current_tab?;
        let tab_state = self.tab_map.get_mut(&current_entity)?;
        Some(&mut tab_state.camera_state)
    }

    fn new_tab_id(&mut self) -> TabId {
        if self.recycled_tab_ids.is_empty() {
            let id = self.new_tab_id;
            self.new_tab_id += 1;
            if self.new_tab_id == TabId::MAX {
                panic!("ran out of tab ids!");
            }
            id
        } else {
            self.recycled_tab_ids.pop_front().unwrap()
        }
    }

    fn recycle_tab_id(&mut self, id: TabId) {
        self.recycled_tab_ids.push_back(id);
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
        self.set_current_tab(*row_entity);
        let tab_state = self.tab_map.get_mut(&row_entity).unwrap();
        tab_state.selected = true;
    }

    fn set_current_tab(&mut self, tab_entity: Entity) {
        self.current_tab = Some(tab_entity);
    }

    fn clear_current_tab(&mut self) {
        self.current_tab = None;
    }

    fn close_all_tabs(&mut self, client: &mut Client) {
        let all_tabs = self.tab_order.clone();
        for entity in all_tabs {
            self.close_tab(client, &entity);
        }
    }

    fn close_all_tabs_except(&mut self, client: &mut Client, row_entity: &Entity) {
        self.close_all_tabs(client);
        if !self.tab_map.contains_key(row_entity) {
            panic!("row entity not in tab map!")
        }
        self.select_tab(row_entity);
    }

    fn close_all_tabs_left_of(&mut self, client: &mut Client, row_entity: &Entity) {
        let tab_state = self.tab_map.get(row_entity).unwrap();
        let order = tab_state.order;
        let mut tabs_to_close: Vec<Entity> = Vec::new();
        for i in 0..order {
            let entity = self.tab_order[i];
            tabs_to_close.push(entity);
        }

        for entity in tabs_to_close {
            self.close_tab(client, &entity);
        }
    }

    fn close_all_tabs_right_of(&mut self, client: &mut Client, row_entity: &Entity) {
        let tab_state = self.tab_map.get(row_entity).unwrap();
        let order = tab_state.order;
        let mut tabs_to_close: Vec<Entity> = Vec::new();
        for i in order + 1..self.tab_order.len() {
            let entity = self.tab_order[i];
            tabs_to_close.push(entity);
        }

        for entity in tabs_to_close {
            self.close_tab(client, &entity);
        }
    }

    fn render_tabs(
        &mut self,
        client: &mut Client,
        ui: &mut Ui,
        file_q: &Query<(&FileSystemEntry, &FileSystemUiState)>,
    ) {
        let mut tab_action = None;

        for row_entity in &self.tab_order {
            let tab_state = self.tab_map.get(row_entity).unwrap();

            let (entry, ui_state) = file_q.get(*row_entity).unwrap();

            let button_response =
                Self::render_tab(ui, row_entity, entry, ui_state, tab_state, &mut tab_action);

            Self::tab_context_menu(button_response, row_entity, &mut tab_action);
        }

        self.execute_tab_action(client, tab_action);
    }

    fn render_tab(
        ui: &mut Ui,
        row_entity: &Entity,
        entry: &FileSystemEntry,
        ui_state: &FileSystemUiState,
        tab_state: &TabState,
        tab_action: &mut Option<TabAction>,
    ) -> Response {
        let x_icon_nohover = "❌";
        let x_icon_hover = "❎";
        let file_name = &*entry.name;
        let full_path = format!("tab_cancel_button:{:?}", row_entity);
        let file_name_str = format!("📃 {}", file_name);

        let file_name_galley =
            WidgetText::from(file_name_str).into_galley(ui, Some(false), 1.0, TextStyle::Button);

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

                let x_icon_galley = WidgetText::from(x_icon_text).into_galley(
                    ui,
                    Some(false),
                    1.0,
                    TextStyle::Button,
                );
                x_icon_galley.paint_with_color_override(
                    ui.painter(),
                    icon_position,
                    text_colors.default,
                );
            }
        }

        tab_response
    }

    fn tab_context_menu(
        button_response: Response,
        row_entity: &Entity,
        tab_action: &mut Option<TabAction>,
    ) {
        // Tab context menu
        button_response.context_menu(|ui| {
            if ui.add(egui::Button::new("Close")).clicked() {
                *tab_action = Some(TabAction::Close(*row_entity));
                ui.close_menu();
            }
            if ui.add(egui::Button::new("Close Other Tabs")).clicked() {
                *tab_action = Some(TabAction::CloseOthers(*row_entity));
                ui.close_menu();
            }
            if ui.add(egui::Button::new("Close All Tabs")).clicked() {
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

    fn execute_tab_action(&mut self, client: &mut Client, tab_action: Option<TabAction>) {
        match tab_action {
            None => {}
            Some(TabAction::Select(row_entity)) => {
                self.select_tab(&row_entity);
            }
            Some(TabAction::Close(row_entity)) => {
                self.close_tab(client, &row_entity);
            }
            Some(TabAction::CloseAll) => {
                self.close_all_tabs(client);
            }
            Some(TabAction::CloseOthers(row_entity)) => {
                self.close_all_tabs_except(client, &row_entity);
            }
            Some(TabAction::CloseLeft(row_entity)) => {
                self.close_all_tabs_left_of(client, &row_entity);
            }
            Some(TabAction::CloseRight(row_entity)) => {
                self.close_all_tabs_right_of(client, &row_entity);
            }
        }
    }
}

pub fn render_tab_bar(ui: &mut Ui, world: &mut World) {
    egui::TopBottomPanel::top("tab_bar").show_inside(ui, |ui| {
        egui::menu::bar(ui, |ui| {
            let mut system_state: SystemState<(
                Client,
                ResMut<TabManager>,
                Query<(&FileSystemEntry, &FileSystemUiState)>,
            )> = SystemState::new(world);
            let (mut client, mut tab_manager, file_q) = system_state.get_mut(world);

            tab_manager.render_tabs(&mut client, ui, &file_q);

            system_state.apply(world);
        });
    });
}
