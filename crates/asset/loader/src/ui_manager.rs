use std::collections::HashMap;

use bevy_ecs::{change_detection::{Mut, Res, ResMut}, event::Event, prelude::World, system::Resource};
use bevy_log::warn;

use asset_id::AssetId;
use clipboard::ClipboardManager;
use input::{CursorIcon, Input};
use math::Vec2;
use render_api::{base::{CpuMaterial, CpuMesh}, components::{Camera, RenderLayer}, resources::{RenderFrame, Time}};
use storage::Storage;
use ui::{NodeId, TextMeasurer, Ui, UiGlobalEvent, UiInputEvent, UiNodeEvent, UiNodeEventHandler};

use crate::{asset_storage::AssetStorage, AssetHandle, AssetManager, IconData, processed_asset_store::ProcessedAssetStore, TypedAssetId, UiData, UiStateData};

#[derive(Resource)]
pub struct UiManager {
    pub uis: AssetStorage<UiData>,
    pub ui_states: HashMap<AssetHandle<UiData>, UiStateData>,
    queued_uis: Vec<AssetHandle<UiData>>,

    queued_ui_node_event_handlers: HashMap<AssetHandle<UiData>, Vec<(String, UiNodeEventHandler)>>,
    ui_global_events: Vec<UiGlobalEvent>,
    ui_node_event_handlers: HashMap<(AssetId, NodeId), UiNodeEventHandler>,
    ui_node_events: Vec<(AssetId, NodeId, UiNodeEvent)>,
    cursor_icon_change: Option<CursorIcon>,
    last_cursor_icon: CursorIcon,
    pub blinkiness: Blinkiness,
}

impl Default for UiManager {
    fn default() -> Self {
        Self {
            uis: AssetStorage::default(),
            ui_states: HashMap::default(),
            queued_uis: Vec::new(),

            ui_global_events: Vec::new(),
            ui_node_events: Vec::new(),
            ui_node_event_handlers: HashMap::new(),
            queued_ui_node_event_handlers: HashMap::new(),
            cursor_icon_change: None,
            last_cursor_icon: CursorIcon::Default,
            blinkiness: Blinkiness::new(),
        }
    }
}

impl UiManager {
    // used as a system
    pub(crate) fn prepare_cursor_change(mut ui_manager: ResMut<Self>) {
        ui_manager.set_cursor_icon_change(None);
    }

    // used as a system
    pub(crate) fn process_ui_global_events(mut ui_manager: ResMut<Self>, mut clipboard_manager: ResMut<ClipboardManager>) {
        ui_manager.process_ui_global_events_impl(&mut clipboard_manager);
    }

    // used as a system
    pub(crate) fn process_ui_node_events(world: &mut World) {
        world.resource_scope(|world, mut ui_manager: Mut<Self>| {
            ui_manager.process_ui_node_events_impl(world);
        });
    }

    // used as a system
    pub(crate) fn process_cursor_change(mut ui_manager: ResMut<Self>, mut input: ResMut<Input>) {
        let Some(cursor_change) = ui_manager.take_cursor_icon_change() else {
            return;
        };
        if cursor_change != ui_manager.get_last_cursor_icon() {
            ui_manager.set_last_cursor_icon(cursor_change);
            input.set_cursor_icon(cursor_change);
        }
    }

    // used as a system
    pub(crate) fn update_blinkiness(mut ui_manager: ResMut<Self>, time: Res<Time>) {
        let elapsed = time.get_elapsed_ms();
        ui_manager.blinkiness.update(elapsed);
    }

    pub fn load(
        &mut self,
        asset_manager: &mut AssetManager,
        asset_data_store: &HashMap<AssetId, Vec<u8>>,
        asset_id: &AssetId,
    ) {
        self.load_impl(asset_manager.get_store_mut(), asset_data_store, asset_id);
    }

    pub fn manual_load_ui(&mut self, asset_id: &AssetId, ui: Ui) {
        let mut dependencies: Vec<(TypedAssetId, TypedAssetId)> = Vec::new();

        let handle = AssetHandle::<UiData>::new(*asset_id);
        if !self.uis.has(&handle) {

            let ui_state_data = UiStateData::from_ui(&ui);
            self.ui_states.insert(handle, ui_state_data);

            let ui_data = UiData::from_ui(ui);
            self.uis.insert(handle, ui_data);

            let ui_data = self.uis.get(&handle).unwrap();
            ui_data.load_dependencies(handle, &mut dependencies);

            self.queued_uis.push(handle);
        }

        if !dependencies.is_empty() {
            for (principal_handle, dependency_handle) in dependencies {
                self.finish_dependency_impl(principal_handle, dependency_handle);
            }
        }
    }

    fn load_impl(
        &mut self,
        store: &mut ProcessedAssetStore,
        asset_data_store: &HashMap<AssetId, Vec<u8>>,
        asset_id: &AssetId,
    ) {
        let mut dependencies: Vec<(TypedAssetId, TypedAssetId)> = Vec::new();

        let handle = AssetHandle::<UiData>::new(*asset_id);
        if !self.uis.has(&handle) {
            let bytes = asset_data_store.get(asset_id).unwrap();
            let ui_data = UiData::from_bytes(bytes);

            {
                let ui = ui_data.get_ui_ref();
                let ui_state_data = UiStateData::from_ui(ui);
                self.ui_states.insert(handle, ui_state_data);
            }

            self.uis.insert(handle, ui_data);

            let ui_data = self.uis.get(&handle).unwrap();
            ui_data.load_dependencies(handle, &mut dependencies);

            self.queued_uis.push(handle);
        }

        if !dependencies.is_empty() {
            for (principal_handle, dependency_handle) in dependencies {
                let dependency_id = dependency_handle.get_id();
                let dependency_type = dependency_handle.get_type();
                store.load(asset_data_store, &dependency_id, &dependency_type);
                self.finish_dependency_impl(principal_handle, dependency_handle);
            }
        }
    }

    fn finish_dependency_impl(
        &mut self,
        principal_typed_id: TypedAssetId,
        dependency_typed_id: TypedAssetId,
    ) {
        let TypedAssetId::Ui(principal_id) = principal_typed_id else { panic!(""); };

        let principal_handle = AssetHandle::<UiData>::new(principal_id);
        let principal_data = self.uis.get_mut(&principal_handle).unwrap();
        principal_data.finish_dependency(dependency_typed_id);
    }

    pub(crate) fn sync_uis(
        &mut self,
        meshes: &mut Storage<CpuMesh>,
        materials: &mut Storage<CpuMaterial>,
    ) {
        if self.queued_uis.is_empty() {
            return;
        }

        let ui_handles = std::mem::take(&mut self.queued_uis);

        for ui_handle in &ui_handles {
            let ui = self.uis.get(ui_handle).unwrap();
            let ui_data = self.ui_states.get_mut(ui_handle).unwrap();
            ui_data.load_cpu_data(ui.get_ui_ref(), meshes, materials);
        }

        self.handle_new_uis(ui_handles);
    }

    fn handle_new_uis(&mut self, new_uis: Vec<AssetHandle<UiData>>) {
        for handle in new_uis {
            if let Some(queued_handlers) =
                self.queued_ui_node_event_handlers.remove(&handle)
            {
                for (id_str, handler) in queued_handlers {
                    let asset_id = handle.asset_id();
                    let ui_data = self.uis.get(&handle).unwrap();
                    let node_id = ui_data
                        .get_ui_ref()
                        .get_node_id_by_id_str(&id_str)
                        .unwrap();
                    self
                        .ui_node_event_handlers
                        .insert((asset_id, node_id), handler);
                }
            }
        }
    }

    pub(crate) fn process_ui_global_events_impl(&mut self, clipboard_manager: &mut ClipboardManager) {
        let global_events = std::mem::take(&mut self.ui_global_events);
        for event in global_events {
            match event {
                UiGlobalEvent::Copied(text) => {
                    clipboard_manager.set_contents(&text);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn process_ui_node_events_impl(&mut self, world: &mut World) {
        if self.ui_node_events.is_empty() {
            return;
        }

        let events = std::mem::take(&mut self.ui_node_events);
        for (ui_asset_id, node_id, event) in events {
            let Some(handler) = self.ui_node_event_handlers.get(&(ui_asset_id, node_id)) else {
                warn!(
                    "no handler for asset_id: {:?}, node_id: {:?}",
                    ui_asset_id, node_id
                );
                continue;
            };

            handler.handle(world, event);
        }
    }

    pub(crate) fn set_cursor_icon_change(&mut self, cursor_icon: Option<CursorIcon>) {
        self.cursor_icon_change = cursor_icon;
    }

    pub(crate) fn take_cursor_icon_change(&mut self) -> Option<CursorIcon> {
        self.cursor_icon_change.take()
    }

    pub(crate) fn get_last_cursor_icon(&self) -> CursorIcon {
        self.last_cursor_icon
    }

    pub(crate) fn set_last_cursor_icon(&mut self, cursor_icon: CursorIcon) {
        self.last_cursor_icon = cursor_icon;
    }

    pub fn register_ui_event<T: Event + Default>(
        &mut self,
        ui_handle: &AssetHandle<UiData>,
        id_str: &str,
    ) {
        let asset_id = ui_handle.asset_id();
        let event_handler = UiNodeEventHandler::new::<T>();

        if let Some(ui_store) = self.uis.get(&ui_handle) {
            let Some(node_id) = ui_store.get_ui_ref().get_node_id_by_id_str(id_str) else {
                panic!("no node_id for id_str: {:?}", id_str);
            };

            self.ui_node_event_handlers
                .insert((asset_id, node_id), event_handler);
        } else {
            if !self.queued_ui_node_event_handlers.contains_key(&ui_handle) {
                self.queued_ui_node_event_handlers
                    .insert(ui_handle.clone(), Vec::new());
            }
            self.queued_ui_node_event_handlers
                .get_mut(&ui_handle)
                .unwrap()
                .push((id_str.to_string(), event_handler));
        }
    }

    pub fn update_ui_viewport(
        &mut self,
        asset_manager: &AssetManager,
        camera: &Camera,
        ui_handle: &AssetHandle<UiData>,
    ) {
        let store = asset_manager.get_store();
        let Some(viewport) = camera.viewport else {
            return;
        };
        let Some(ui_state_data) = self.ui_states.get_mut(ui_handle) else {
            warn!("ui data not loaded 1: {:?}", ui_handle.asset_id());
            return;
        };

        let ui_state = ui_state_data.get_ui_state_mut();

        ui_state.update_viewport(&viewport);

        let needs_to_recalc = ui_state.needs_to_recalculate_layout();

        if needs_to_recalc {
            let Some(ui_data) = self.uis.get(ui_handle) else {
                warn!("ui data not loaded 1: {:?}", ui_handle.asset_id());
                return;
            };
            let icon_handle = ui_data.get_icon_handle();
            self.recalculate_ui_layout(store, ui_handle, &icon_handle);
        }
    }

    fn recalculate_ui_layout(&mut self, store: &ProcessedAssetStore, ui_handle: &AssetHandle<UiData>, icon_handle: &AssetHandle<IconData>) {
        let Some(icon_data) = store.icons.get(&icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);

        let Some(ui_data) = self.uis.get_mut(ui_handle) else {
            return;
        };
        let Some(ui_state_data) = self.ui_states.get_mut(ui_handle) else {
            return;
        };

        ui_state_data.get_ui_state_mut().recalculate_layout(ui_data.get_ui_mut(), &text_measurer);
    }

    pub fn update_ui_input(
        &mut self,
        asset_manager: &AssetManager,
        ui_handle: &AssetHandle<UiData>,
        mouse_position: Option<Vec2>,
        ui_input_events: Vec<UiInputEvent>,
    ) {
        let store = asset_manager.get_store();
        let Some(ui_data) = self.uis.get(ui_handle) else {
            warn!("ui data not loaded 1: {:?}", ui_handle.asset_id());
            return;
        };
        let Some(ui_state_data) = self.ui_states.get_mut(ui_handle) else {
            warn!("ui data not loaded 1: {:?}", ui_handle.asset_id());
            return;
        };
        let icon_handle = ui_data.get_icon_handle();
        let Some(icon_data) = store.icons.get(&icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);
        let ui_state = ui_state_data.get_ui_state_mut();
        ui_state.receive_input(ui_data.get_ui_ref(), &text_measurer, mouse_position, ui_input_events);

        // get any global events
        let mut global_events: Vec<UiGlobalEvent> = ui_state.take_global_events();
        self.ui_global_events.append(&mut global_events);

        // get any node events
        let mut events: Vec<(AssetId, NodeId, UiNodeEvent)> = ui_state
            .take_node_events()
            .iter()
            .map(|(node_id, event)| (ui_handle.asset_id(), *node_id, event.clone()))
            .collect();

        self.ui_node_events.append(&mut events);

        // get cursor icon change
        let new_cursor_icon = ui_state.get_cursor_icon();
        if new_cursor_icon != self.last_cursor_icon {
            self.cursor_icon_change = Some(new_cursor_icon);
        }
    }
}

pub struct UiTextMeasurer<'a> {
    icon_data: &'a IconData,
}

impl<'a> UiTextMeasurer<'a> {
    pub fn new(icon_data: &'a IconData) -> Self {
        Self { icon_data }
    }
}

impl<'a> TextMeasurer for UiTextMeasurer<'a> {

    fn get_raw_char_width(&self, subimage: usize) -> f32 {
        if subimage == 0 {
            return 40.0;
        }
        self.icon_data.get_frame_width(subimage).unwrap_or(0.0)
    }

    fn get_raw_char_height(&self, _subimage: usize) -> f32 {
        200.0
    }
}

pub struct Blinkiness {
    value: bool,
    accumulated_ms: f32,
}

impl Blinkiness {
    pub fn new() -> Self {
        Self {
            value: true,
            accumulated_ms: 0.0,
        }
    }

    pub fn update(&mut self, delta_ms: f32) {
        self.accumulated_ms += delta_ms;
        if self.accumulated_ms >= 500.0 {
            self.value = !self.value;
            self.accumulated_ms = 0.0;
        }
    }

    pub fn enabled(&self) -> bool {
        self.value
    }
}