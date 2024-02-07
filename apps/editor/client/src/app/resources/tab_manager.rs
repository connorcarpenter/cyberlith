use std::collections::{HashMap, VecDeque};

use bevy_ecs::{
    prelude::{Entity, Resource},
    query::{With, Without},
    system::{Query, Res, ResMut, SystemState},
    world::{Mut, World},
};

use naia_bevy_client::Client;

use render_api::{
    base::{Color, CpuMaterial},
    components::Visibility,
};
use render_egui::{
    egui,
    egui::{vec2, Id, NumExt, Rect, Response, Rounding, Sense, Stroke, TextStyle, Ui, WidgetText},
};
use storage::{Assets, Handle};

use editor_proto::{
    channels::TabActionChannel,
    components::{
        BackgroundSkinColor, ChangelistStatus, FaceColor, FileExtension, FileSystemEntry, IconFace,
        NetTransformEntityType, PaletteColor, SkinOrSceneEntity,
    },
    messages::{TabCloseMessage, TabOpenMessage},
    types::TabId,
};

use crate::app::{
    components::{
        file_system::FileSystemUiState, Edge2dLocal, Face3dLocal, FaceIcon2d, IconLocalFace,
        LocalShape, OwnedByFileLocal, Vertex2d,
    },
    plugin::Main,
    resources::{
        action::{
            animation::AnimAction, icon::IconAction, model::ModelAction, palette::PaletteAction,
            shape::ShapeAction, skin::SkinAction, TabActionStack,
        },
        camera_manager::CameraManager,
        camera_state::CameraState,
        canvas::Canvas,
        face_manager::FaceManager,
        file_manager::FileManager,
        icon_manager::IconManager,
        input::InputManager,
        model_manager::ModelManager,
        palette_manager::PaletteManager,
        shape_data::CanvasShape,
        shape_manager::ShapeManager,
        skin_manager::SkinManager,
    },
    ui::{
        widgets::colors::{
            FILE_ROW_COLORS_HOVER, FILE_ROW_COLORS_SELECTED, FILE_ROW_COLORS_UNSELECTED,
            TEXT_COLORS_HOVER, TEXT_COLORS_SELECTED, TEXT_COLORS_UNSELECTED,
        },
        UiState,
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
    resync_shape_colors: bool,
    // entity is file entity here
    tab_map: HashMap<Entity, TabState>,
    tab_order: Vec<Entity>,
    new_tab_id: TabId,
    recycled_tab_ids: VecDeque<TabId>,

    content_has_focus: bool,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
            current_tab: None,
            last_tab: None,
            resync_tab_ownership: false,
            resync_shape_colors: false,
            tab_map: HashMap::new(),
            tab_order: Vec::new(),
            new_tab_id: 0,
            recycled_tab_ids: VecDeque::new(),

            content_has_focus: false,
        }
    }
}

impl TabManager {
    pub fn sync_tabs(world: &mut World) {
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.on_sync_tabs(world);
            tab_manager.on_sync_tab_ownership(world);
            tab_manager.on_sync_shape_colors(world);
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
        )> = SystemState::new(world);
        let (mut canvas, file_manager, mut input_manager, mut camera_manager) =
            system_state.get_mut(world);

        let mut canvas_is_visible = false;
        if let Some(current_file_entity) = self.current_tab {
            let current_file_type = file_manager.get_file_type(&current_file_entity);
            canvas_is_visible = match current_file_type {
                FileExtension::Palette => false,
                FileExtension::Skel
                | FileExtension::Mesh
                | FileExtension::Anim
                | FileExtension::Skin
                | FileExtension::Model
                | FileExtension::Scene
                | FileExtension::Icon => true,
                _ => false,
            };
        }

        if canvas_is_visible {
            canvas.set_visibility(true);
            self.set_focus(true);
        } else {
            canvas.set_visibility(false);
            self.set_focus(false);
        }

        input_manager.deselect_shape(&mut canvas);
        if let Some(tab_state) = self.current_tab_state_mut() {
            if let Some(Some((entity, shape))) = tab_state.selected_shape_2d.take() {
                input_manager.select_shape(&mut canvas, &entity, shape);
            }
        }

        camera_manager.recalculate_3d_view();
        self.resync_tab_ownership();
        self.resync_shape_colors();
    }

    pub fn resync_tab_ownership(&mut self) {
        self.resync_tab_ownership = true;
    }

    pub fn resync_shape_colors(&mut self) {
        self.resync_shape_colors = true;
    }

    pub fn on_sync_tab_ownership(&mut self, world: &mut World) {
        if !self.resync_tab_ownership {
            return;
        }

        self.resync_tab_ownership = false;

        let mut system_state: SystemState<(
            Res<FileManager>,
            ResMut<Canvas>,
            ResMut<UiState>,
            Query<(&mut Visibility, &OwnedByFileLocal)>,
        )> = SystemState::new(world);
        let (file_manager, mut canvas, mut ui_state, mut visibility_q) =
            system_state.get_mut(world);

        if let Some(current_tab_file_entity) = self.current_tab_entity() {
            for (mut visibility, owned_by_tab) in visibility_q.iter_mut() {
                visibility.visible = ShapeManager::is_owned_by_file(
                    &file_manager,
                    current_tab_file_entity,
                    Some(&owned_by_tab.file_entity),
                );
            }
        }

        ui_state.canvas_coords = None;

        canvas.queue_resync_shapes();
    }

    pub fn on_sync_shape_colors(&mut self, world: &mut World) {
        if !self.resync_shape_colors {
            return;
        }

        self.resync_shape_colors = false;

        if let Some(current_file_entity) = self.current_tab {
            let file_ext = world
                .get_resource::<FileManager>()
                .unwrap()
                .get_file_type(&current_file_entity);
            file_ext_specific_sync_tabs_shape_colors(&file_ext, &current_file_entity, world);
        }
    }

    pub fn has_focus(&self) -> bool {
        self.content_has_focus
    }

    pub fn set_focus(&mut self, focus: bool) {
        self.content_has_focus = focus;
    }

    pub fn open_tab(
        &mut self,
        client: &mut Client<Main>,
        row_entity: &Entity,
        file_ext: FileExtension,
    ) {
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

    pub fn close_tab(&mut self, client: &mut Client<Main>, row_entity: &Entity) {
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

    pub fn current_tab_execute_skin_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        action: SkinAction,
    ) {
        let current_tab_entity = *self.current_tab_entity().unwrap();
        let tab_state = self.current_tab_state_mut().unwrap();
        tab_state.action_stack.execute_skin_action(
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

    pub fn current_tab_execute_palette_action(
        &mut self,
        world: &mut World,
        palette_manager: &mut PaletteManager,
        action: PaletteAction,
    ) {
        let tab_state = self.current_tab_state_mut().unwrap();
        tab_state
            .action_stack
            .execute_palette_action(world, palette_manager, action);
    }

    pub fn current_tab_execute_model_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        action: ModelAction,
    ) {
        let current_tab_entity = *self.current_tab_entity().unwrap();
        let tab_state = self.current_tab_state_mut().unwrap();
        tab_state.action_stack.execute_model_action(
            world,
            input_manager,
            current_tab_entity,
            action,
        );
    }

    pub fn current_tab_execute_icon_action(
        &mut self,
        world: &mut World,
        icon_manager: &mut IconManager,
        action: IconAction,
    ) {
        let current_tab_entity = *self.current_tab_entity().unwrap();
        let tab_state = self.current_tab_state_mut().unwrap();
        tab_state
            .action_stack
            .execute_icon_action(world, icon_manager, current_tab_entity, action);
    }

    pub fn current_tab_camera_state(&self) -> Option<&CameraState> {
        let current_entity = self.current_tab?;
        let tab_state = self.tab_map.get(&current_entity)?;
        Some(&tab_state.camera_state)
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

    fn close_all_tabs(&mut self, client: &mut Client<Main>) {
        let all_tabs = self.tab_order.clone();
        for entity in all_tabs {
            self.close_tab(client, &entity);
        }
    }

    fn close_all_tabs_except(&mut self, client: &mut Client<Main>, row_entity: &Entity) {
        self.close_all_tabs(client);
        if !self.tab_map.contains_key(row_entity) {
            panic!("row entity not in tab map!")
        }
        self.select_tab(row_entity);
    }

    fn close_all_tabs_left_of(&mut self, client: &mut Client<Main>, row_entity: &Entity) {
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

    fn close_all_tabs_right_of(&mut self, client: &mut Client<Main>, row_entity: &Entity) {
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
        client: &mut Client<Main>,
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

    fn execute_tab_action(&mut self, client: &mut Client<Main>, tab_action: Option<TabAction>) {
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
                Client<Main>,
                ResMut<TabManager>,
                Query<(&FileSystemEntry, &FileSystemUiState)>,
            )> = SystemState::new(world);
            let (mut client, mut tab_manager, file_q) = system_state.get_mut(world);

            tab_manager.render_tabs(&mut client, ui, &file_q);

            system_state.apply(world);
        });
    });
}

fn file_ext_specific_sync_tabs_shape_colors(
    file_ext: &FileExtension,
    current_file_entity: &Entity,
    world: &mut World,
) {
    match file_ext {
        FileExtension::Skin => {
            let mut system_state: SystemState<(
                Client<Main>,
                Res<FileManager>,
                Res<FaceManager>,
                Res<PaletteManager>,
                Res<SkinManager>,
                ResMut<Assets<CpuMaterial>>,
                Query<&mut Handle<CpuMaterial>, (With<Edge2dLocal>, Without<LocalShape>)>,
                Query<
                    &mut Handle<CpuMaterial>,
                    (With<FaceIcon2d>, Without<Edge2dLocal>, Without<Face3dLocal>),
                >,
                Query<
                    (Entity, &mut Handle<CpuMaterial>, &OwnedByFileLocal),
                    (With<Face3dLocal>, Without<Edge2dLocal>, Without<FaceIcon2d>),
                >,
                Query<&PaletteColor>,
                Query<&BackgroundSkinColor>,
                Query<&FaceColor>,
            )> = SystemState::new(world);
            let (
                client,
                file_manager,
                face_manager,
                palette_manager,
                skin_manager,
                mut materials,
                mut edge_2d_q,
                mut face_2d_q,
                mut face_3d_q,
                palette_color_q,
                bckg_color_q,
                face_color_q,
            ) = system_state.get_mut(world);

            let gray_mat_handle = materials.add(CpuMaterial::new(Color::LIGHT_GRAY, 0.0, 0.0, 0.0));
            for mut mat_handle in edge_2d_q.iter_mut() {
                *mat_handle = gray_mat_handle;
            }

            set_face_3d_colors(
                current_file_entity,
                &client,
                &file_manager,
                &palette_manager,
                &skin_manager,
                &mut materials,
                &mut face_3d_q,
                &palette_color_q,
                &bckg_color_q,
                &face_color_q,
                &mut Some((&face_manager, &mut face_2d_q)),
            );
        }
        FileExtension::Model | FileExtension::Scene => {
            let mut system_state: SystemState<(
                Client<Main>,
                Res<FileManager>,
                Res<PaletteManager>,
                Res<SkinManager>,
                Res<ModelManager>,
                ResMut<Assets<CpuMaterial>>,
                Query<
                    (Entity, &mut Handle<CpuMaterial>, &OwnedByFileLocal),
                    (With<Face3dLocal>, Without<Edge2dLocal>, Without<FaceIcon2d>),
                >,
                Query<&PaletteColor>,
                Query<&BackgroundSkinColor>,
                Query<&FaceColor>,
                Query<&SkinOrSceneEntity>,
            )> = SystemState::new(world);
            let (
                client,
                file_manager,
                palette_manager,
                skin_manager,
                model_manager,
                mut materials,
                mut face_3d_q,
                palette_color_q,
                bckg_color_q,
                face_color_q,
                skin_or_scene_q,
            ) = system_state.get_mut(world);

            set_face_3d_colors_recursive(
                current_file_entity,
                &client,
                &file_manager,
                &palette_manager,
                &skin_manager,
                &model_manager,
                &mut materials,
                &mut face_3d_q,
                &palette_color_q,
                &bckg_color_q,
                &face_color_q,
                &skin_or_scene_q,
            );
        }
        FileExtension::Icon => {
            let mut system_state: SystemState<(
                Client<Main>,
                Res<FileManager>,
                Res<IconManager>,
                Res<PaletteManager>,
                ResMut<Assets<CpuMaterial>>,
                Query<&mut Handle<CpuMaterial>, (With<IconLocalFace>, Without<IconFace>)>,
                Query<
                    (
                        Entity,
                        &IconFace,
                        &mut Handle<CpuMaterial>,
                        &OwnedByFileLocal,
                    ),
                    Without<IconLocalFace>,
                >,
                Query<&PaletteColor>,
            )> = SystemState::new(world);
            let (
                client,
                file_manager,
                icon_manager,
                palette_manager,
                mut materials,
                mut local_face_q,
                mut net_face_q,
                palette_color_q,
            ) = system_state.get_mut(world);

            set_icon_face_colors(
                current_file_entity,
                &client,
                &file_manager,
                &palette_manager,
                &mut materials,
                &mut net_face_q,
                &palette_color_q,
                &mut Some((&icon_manager, &mut local_face_q)),
            );
        }
        _ => {
            let mut system_state: SystemState<(
                ResMut<Assets<CpuMaterial>>,
                Query<&mut Handle<CpuMaterial>, (With<Edge2dLocal>, Without<LocalShape>)>,
                Query<
                    &mut Handle<CpuMaterial>,
                    (With<FaceIcon2d>, Without<Edge2dLocal>, Without<Face3dLocal>),
                >,
                Query<
                    (Entity, &mut Handle<CpuMaterial>),
                    (With<Face3dLocal>, Without<Edge2dLocal>, Without<FaceIcon2d>),
                >,
            )> = SystemState::new(world);
            let (mut materials, mut edge_2d_q, mut face_2d_q, mut face_3d_q) =
                system_state.get_mut(world);

            let enabled_mat_handle =
                materials.add(CpuMaterial::new(Vertex2d::ENABLED_COLOR, 0.0, 0.0, 0.0));

            for mut mat_handle in edge_2d_q.iter_mut() {
                *mat_handle = enabled_mat_handle;
            }
            for mut mat_handle in face_2d_q.iter_mut() {
                *mat_handle = enabled_mat_handle;
            }
            for (_, mut mat_handle) in face_3d_q.iter_mut() {
                *mat_handle = enabled_mat_handle;
            }
        }
    }
}

fn set_face_3d_colors_recursive(
    current_file_entity: &Entity,
    client: &Client<Main>,
    file_manager: &FileManager,
    palette_manager: &PaletteManager,
    skin_manager: &SkinManager,
    model_manager: &ModelManager,
    materials: &mut Assets<CpuMaterial>,
    face_3d_q: &mut Query<
        (Entity, &mut Handle<CpuMaterial>, &OwnedByFileLocal),
        (With<Face3dLocal>, Without<Edge2dLocal>, Without<FaceIcon2d>),
    >,
    palette_color_q: &Query<&PaletteColor>,
    bckg_color_q: &Query<&BackgroundSkinColor>,
    face_color_q: &Query<&FaceColor>,
    skin_or_scene_q: &Query<&SkinOrSceneEntity>,
) {
    let Some(net_transform_entities) = model_manager.file_transform_entities(current_file_entity) else {
        return;
    };
    for net_transform_entity in net_transform_entities {
        let Ok(skin_or_scene_entity) = skin_or_scene_q.get(net_transform_entity) else {
            continue;
        };
        match *skin_or_scene_entity.value_type {
            NetTransformEntityType::Skin => {
                let skin_file_entity: Entity = skin_or_scene_entity.value.get(client).unwrap();
                set_face_3d_colors(
                    &skin_file_entity,
                    client,
                    file_manager,
                    palette_manager,
                    skin_manager,
                    materials,
                    face_3d_q,
                    palette_color_q,
                    bckg_color_q,
                    face_color_q,
                    &mut None,
                );
            }
            NetTransformEntityType::Scene => {
                let scene_file_entity = skin_or_scene_entity.value.get(client).unwrap();
                set_face_3d_colors_recursive(
                    &scene_file_entity,
                    client,
                    file_manager,
                    palette_manager,
                    skin_manager,
                    model_manager,
                    materials,
                    face_3d_q,
                    palette_color_q,
                    bckg_color_q,
                    face_color_q,
                    skin_or_scene_q,
                );
            }
            _ => {
                panic!("invalid");
            }
        }
    }
}

fn set_face_3d_colors(
    skin_file_entity: &Entity,
    client: &Client<Main>,
    file_manager: &FileManager,
    palette_manager: &PaletteManager,
    skin_manager: &SkinManager,
    materials: &mut Assets<CpuMaterial>,
    face_3d_q: &mut Query<
        (Entity, &mut Handle<CpuMaterial>, &OwnedByFileLocal),
        (With<Face3dLocal>, Without<Edge2dLocal>, Without<FaceIcon2d>),
    >,
    palette_color_q: &Query<&PaletteColor>,
    bckg_color_q: &Query<&BackgroundSkinColor>,
    face_color_q: &Query<&FaceColor>,
    face_2d_opt: &mut Option<(
        &FaceManager,
        &mut Query<
            &mut Handle<CpuMaterial>,
            (With<FaceIcon2d>, Without<Edge2dLocal>, Without<Face3dLocal>),
        >,
    )>,
) {
    // get background color
    let background_index = skin_manager.background_color_index(
        client,
        skin_file_entity,
        bckg_color_q,
        palette_color_q,
    );
    let Some(palette_file_entity) = file_manager.file_get_dependency(skin_file_entity, FileExtension::Palette) else {
        return;
    };
    let Some(mesh_file_entity) = file_manager.file_get_dependency(skin_file_entity, FileExtension::Mesh) else {
        return;
    };
    let Some(colors) = palette_manager.get_file_colors(&palette_file_entity) else {
        panic!("no colors for given file");
    };
    let Some(background_color_entity) = colors.get(background_index).unwrap() else {
        return;
    };
    let background_color = palette_color_q.get(*background_color_entity).unwrap();
    let bckg_mat_handle = materials.add(CpuMaterial::new(
        Color::new(
            *background_color.r,
            *background_color.g,
            *background_color.b,
        ),
        0.0,
        0.0,
        0.0,
    ));

    for (face_3d_entity, mut face_3d_material, owned_by_file) in face_3d_q.iter_mut() {
        if owned_by_file.file_entity != mesh_file_entity {
            continue;
        }

        let new_mat_handle;
        if let Some(face_color_entity) = skin_manager.face_to_color_entity(&face_3d_entity) {
            // use face color
            let face_color = face_color_q.get(*face_color_entity).unwrap();
            let palette_color_entity = face_color.palette_color_entity.get(client).unwrap();
            let palette_color = palette_color_q.get(palette_color_entity).unwrap();
            new_mat_handle = materials.add(CpuMaterial::new(
                Color::new(*palette_color.r, *palette_color.g, *palette_color.b),
                0.0,
                0.0,
                0.0,
            ));
        } else {
            // use background color
            new_mat_handle = bckg_mat_handle;
        }

        *face_3d_material = new_mat_handle;

        if let Some((face_manager, face_2d_q)) = face_2d_opt {
            let face_2d_entity = face_manager.face_entity_3d_to_2d(&face_3d_entity).unwrap();
            let mut face_2d_material = face_2d_q.get_mut(face_2d_entity).unwrap();
            *face_2d_material = new_mat_handle;
        }
    }
}

fn set_icon_face_colors(
    icon_file_entity: &Entity,
    client: &Client<Main>,
    file_manager: &FileManager,
    palette_manager: &PaletteManager,
    materials: &mut Assets<CpuMaterial>,
    net_face_q: &mut Query<
        (
            Entity,
            &IconFace,
            &mut Handle<CpuMaterial>,
            &OwnedByFileLocal,
        ),
        Without<IconLocalFace>,
    >,
    palette_color_q: &Query<&PaletteColor>,
    local_face_opt: &mut Option<(
        &IconManager,
        &mut Query<&mut Handle<CpuMaterial>, (With<IconLocalFace>, Without<IconFace>)>,
    )>,
) {
    let Some(palette_file_entity) = file_manager.file_get_dependency(icon_file_entity, FileExtension::Palette) else {
        return;
    };
    if !palette_manager.has_file_colors(&palette_file_entity) {
        panic!("no colors for given file");
    };

    for (net_face_entity, icon_face, mut net_face_material, owned_by_file) in net_face_q.iter_mut()
    {
        if owned_by_file.file_entity != *icon_file_entity {
            continue;
        }

        let new_mat_handle;

        let palette_color_entity = icon_face.palette_color_entity.get(client).unwrap();
        let palette_color = palette_color_q.get(palette_color_entity).unwrap();
        new_mat_handle = materials.add(CpuMaterial::new(
            Color::new(*palette_color.r, *palette_color.g, *palette_color.b),
            0.0,
            0.0,
            0.0,
        ));

        *net_face_material = new_mat_handle;

        if let Some((icon_manager, local_face_q)) = local_face_opt {
            let local_face_entity = icon_manager
                .face_entity_net_to_local(&net_face_entity)
                .unwrap();
            let mut local_face_material = local_face_q.get_mut(local_face_entity).unwrap();
            *local_face_material = new_mat_handle;
        }
    }
}
