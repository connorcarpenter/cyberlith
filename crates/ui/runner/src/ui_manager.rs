use std::collections::HashMap;

use bevy_ecs::{
    change_detection::{Mut, Res, ResMut},
    event::Event,
    prelude::World,
    system::Resource,
};

use asset_id::AssetId;
use asset_loader::{
    AssetHandle, AssetManager, IconData, ProcessedAssetStore, TypedAssetId, UiTextMeasurer,
};
use clipboard::ClipboardManager;
use input::{CursorIcon, Input};
use logging::{info, warn};
use math::Vec2;
use render_api::{
    shapes::UnitSquare,
    base::{CpuMaterial, CpuMesh},
    components::{Camera, RenderLayer},
    resources::Time,
};
use storage::{Handle, Storage};
use ui_input::{
    ui_receive_input, UiGlobalEvent, UiInputEvent, UiInputState, UiManagerTrait, UiNodeEvent,
    UiNodeEventHandler,
};
use ui_runner_config::{NodeId, UiRuntimeConfig, NodeStore};
use ui_state::{NodeActiveState, UiState};

use crate::{
    config::{UiNode, WidgetKind, ValidationType}, handle::UiHandle, runtime::UiRuntime, state_globals::StateGlobals,
};

#[derive(Resource)]
pub struct UiManager {
    active_ui: Option<UiHandle>,

    // this is the RenderLayer that RenderLayer::UI seeks to mirror
    target_render_layer: Option<RenderLayer>,

    pub ui_runtimes: HashMap<UiHandle, UiRuntime>,
    queued_uis: Vec<UiHandle>,

    queued_ui_node_event_handlers: HashMap<UiHandle, Vec<(String, UiNodeEventHandler)>>,
    ui_global_events: Vec<UiGlobalEvent>,
    ui_node_event_handlers: HashMap<(AssetId, NodeId), UiNodeEventHandler>,
    ui_node_events: Vec<(AssetId, NodeId, UiNodeEvent)>,
    cursor_icon_change: Option<CursorIcon>,
    last_cursor_icon: CursorIcon,
    pub blinkiness: Blinkiness,

    globals: StateGlobals,
    input_state: UiInputState,

    recalc_layout: bool,
}

impl Default for UiManager {
    fn default() -> Self {
        Self {
            active_ui: None,
            target_render_layer: None,

            ui_runtimes: HashMap::new(),
            queued_uis: Vec::new(),

            ui_global_events: Vec::new(),
            ui_node_events: Vec::new(),
            ui_node_event_handlers: HashMap::new(),
            queued_ui_node_event_handlers: HashMap::new(),
            cursor_icon_change: None,
            last_cursor_icon: CursorIcon::Default,
            blinkiness: Blinkiness::new(),

            globals: StateGlobals::new(),
            input_state: UiInputState::new(),

            recalc_layout: false,
        }
    }
}

impl UiManagerTrait for UiManager {
    fn ui_input_state(&self) -> &UiInputState {
        &self.input_state
    }

    fn ui_input_state_mut(&mut self) -> &mut UiInputState {
        &mut self.input_state
    }

    fn ui_state(&self, asset_id: &AssetId) -> &UiState {
        let ui_handle = UiHandle::new(*asset_id);
        self.ui_runtimes.get(&ui_handle).unwrap().ui_state_ref()
    }

    fn ui_state_mut(&mut self, asset_id: &AssetId) -> &mut UiState {
        let ui_handle = UiHandle::new(*asset_id);
        self.ui_runtimes.get_mut(&ui_handle).unwrap().ui_state_mut()
    }

    fn root_ui_asset_id(&self) -> AssetId {
        self.active_ui.unwrap().asset_id()
    }

    fn nodes_iter(&self, asset_id: &AssetId) -> std::collections::btree_map::Iter<'_, NodeId, UiNode> {
        let ui_handle = UiHandle::new(*asset_id);
        self.ui_runtimes
            .get(&ui_handle)
            .unwrap()
            .ui_config_ref()
            .nodes_iter()
    }

    fn ui_config(&self, asset_id: &AssetId) -> Option<&UiRuntimeConfig> {
        let ui_handle = UiHandle::new(*asset_id);
        self.ui_runtimes.get(&ui_handle).map(|r| r.ui_config_ref())
    }

    fn textbox_receive_hover(
        &mut self,
        asset_id: &AssetId,
        node_id: &NodeId,
        bounds: (f32, f32, f32, f32),
        mouse_x: f32,
        mouse_y: f32,
    ) -> bool {
        if let Some(ui_runtime) = self.ui_runtimes.get_mut(&UiHandle::new(*asset_id)) {
            ui_runtime.textbox_receive_hover(node_id, bounds, mouse_x, mouse_y)
        } else {
            false
        }
    }
}

impl UiManager {
    pub(crate) fn update_ui_state(&mut self, delta_ms: f32) {
        for (_, ui) in self.ui_runtimes.iter_mut() {
            ui.update_state(delta_ms);
        }
    }

    // used as a system
    pub(crate) fn startup(mut ui_manager: ResMut<Self>, mut meshes: ResMut<Storage<CpuMesh>>) {
        let mesh_handle = meshes.add(UnitSquare);
        ui_manager.globals.set_box_mesh_handle(mesh_handle);
    }

    // used as a system
    pub(crate) fn sync_assets(
        mut ui_manager: ResMut<Self>,
        mut materials: ResMut<Storage<CpuMaterial>>,
    ) {
        ui_manager.sync_uis(&mut materials);
    }

    // used as a system
    pub fn prepare_cursor_change(mut ui_manager: ResMut<Self>) {
        ui_manager.set_cursor_icon_change(None);
    }

    // used as a system
    pub fn process_ui_global_events(
        mut ui_manager: ResMut<Self>,
        mut clipboard_manager: ResMut<ClipboardManager>,
    ) {
        ui_manager.process_ui_global_events_impl(&mut clipboard_manager);
    }

    // used as a system
    pub fn process_ui_node_events(world: &mut World) {
        world.resource_scope(|world, mut ui_manager: Mut<Self>| {
            ui_manager.process_ui_node_events_impl(world);
        });
    }

    // used as a system
    pub fn process_cursor_change(mut ui_manager: ResMut<Self>, mut input: ResMut<Input>) {
        let Some(cursor_change) = ui_manager.take_cursor_icon_change() else {
            return;
        };
        if cursor_change != ui_manager.get_last_cursor_icon() {
            ui_manager.set_last_cursor_icon(cursor_change);
            input.set_cursor_icon(cursor_change);
        }
    }

    // used as a system
    pub fn update_blinkiness(mut ui_manager: ResMut<Self>, time: Res<Time>) {
        let elapsed = time.get_elapsed_ms();
        ui_manager.blinkiness.update(elapsed);
    }

    pub fn load(
        &mut self,
        asset_manager: &mut AssetManager,
        asset_data_store: &HashMap<AssetId, Vec<u8>>,
        asset_id: &AssetId,
    ) {
        let store = asset_manager.get_store_mut();
        let mut dependencies: Vec<(TypedAssetId, TypedAssetId)> = Vec::new();

        let handle = UiHandle::new(*asset_id);
        if !self.ui_runtimes.contains_key(&handle) {
            let bytes = asset_data_store.get(asset_id).unwrap();
            let Ok(runtime) = UiRuntime::load_from_bytes(bytes) else {
                panic!(
                    "failed to read UiRuntime from bytes at asset_id: {:?}",
                    asset_id
                );
            };

            self.ui_runtimes.insert(handle, runtime);

            let runtime = self.ui_runtimes.get(&handle).unwrap();
            runtime.load_dependencies(handle, &mut dependencies);

            self.queue_ui_for_sync(&handle);
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

    pub fn manual_load_ui_config(
        &mut self,
        asset_id: &AssetId,
        ui_config: UiRuntimeConfig,
    ) -> UiHandle {
        let mut dependencies: Vec<(TypedAssetId, TypedAssetId)> = Vec::new();

        let handle = UiHandle::new(*asset_id);
        if !self.ui_runtimes.contains_key(&handle) {
            let runtime = UiRuntime::load_from_config(asset_id, ui_config);
            self.ui_runtimes.insert(handle, runtime);

            let runtime = self.ui_runtimes.get(&handle).unwrap();
            runtime.load_dependencies(handle, &mut dependencies);

            self.queue_ui_for_sync(&handle);
        }

        if !dependencies.is_empty() {
            for (principal_handle, dependency_handle) in dependencies {
                self.finish_dependency_impl(principal_handle, dependency_handle);
            }
        }

        handle
    }

    pub fn enable_ui(&mut self, handle: &UiHandle) {
        self.active_ui = Some(*handle);
    }

    pub fn disable_ui(&mut self) {
        self.active_ui = None;
    }

    pub fn active_ui(&self) -> Option<UiHandle> {
        self.active_ui
    }

    // this is the RenderLayer that RenderLayer::UI seeks to mirror
    pub fn target_render_layer(&self) -> Option<RenderLayer> {
        self.target_render_layer
    }

    // this is the RenderLayer that RenderLayer::UI seeks to mirror
    pub fn set_target_render_layer(&mut self, render_layer: RenderLayer) {
        self.target_render_layer = Some(render_layer);
    }

    pub fn get_text_icon_handle(&self) -> Option<&AssetHandle<IconData>> {
        self.globals.get_text_icon_handle()
    }

    pub fn set_text_icon_handle(&mut self, asset_id: AssetId) {
        self.globals.set_text_icon_handle(asset_id);
    }

    pub fn get_eye_icon_handle(&self) -> Option<&AssetHandle<IconData>> {
        self.globals.get_eye_icon_handle()
    }

    pub fn set_eye_icon_handle(&mut self, asset_id: AssetId) {
        self.globals.set_eye_icon_handle(asset_id);
    }

    fn finish_dependency_impl(
        &mut self,
        principal_typed_id: TypedAssetId,
        dependency_typed_id: TypedAssetId,
    ) {
        let TypedAssetId::Ui(principal_id) = principal_typed_id else {
            panic!("");
        };

        let principal_handle = UiHandle::new(principal_id);
        let principal_data = self.ui_runtimes.get_mut(&principal_handle).unwrap();
        principal_data.finish_dependency(dependency_typed_id);
    }

    pub fn queue_ui_for_sync(&mut self, handle: &UiHandle) {
        self.queued_uis.push(*handle);
    }

    pub fn sync_uis(&mut self, materials: &mut Storage<CpuMaterial>) {
        if self.queued_uis.is_empty() {
            return;
        }

        let ui_handles = std::mem::take(&mut self.queued_uis);

        for ui_handle in &ui_handles {
            let ui = self.ui_runtimes.get_mut(ui_handle).unwrap();
            ui.load_cpu_data(ui_handle, materials);
        }

        self.handle_new_uis(ui_handles);
    }

    fn handle_new_uis(&mut self, new_uis: Vec<UiHandle>) {
        for handle in new_uis {
            if let Some(queued_handlers) = self.queued_ui_node_event_handlers.remove(&handle) {
                for (id_str, handler) in queued_handlers {
                    let asset_id = handle.asset_id();
                    let ui_runtime = self.ui_runtimes.get(&handle).unwrap();
                    let Some(node_id) = ui_runtime.get_node_id_by_id_str(&id_str) else {
                        panic!("no node_id for id_str: {:?}", id_str);
                    };
                    self.ui_node_event_handlers
                        .insert((asset_id, node_id), handler);
                }
            }
        }
    }

    pub fn process_ui_global_events_impl(&mut self, clipboard_manager: &mut ClipboardManager) {
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

    pub fn process_ui_node_events_impl(&mut self, world: &mut World) {
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

    pub fn set_cursor_icon_change(&mut self, cursor_icon: Option<CursorIcon>) {
        self.cursor_icon_change = cursor_icon;
    }

    pub fn take_cursor_icon_change(&mut self) -> Option<CursorIcon> {
        self.cursor_icon_change.take()
    }

    pub fn get_last_cursor_icon(&self) -> CursorIcon {
        self.last_cursor_icon
    }

    pub fn set_last_cursor_icon(&mut self, cursor_icon: CursorIcon) {
        self.last_cursor_icon = cursor_icon;
    }

    pub fn ui_has_node_with_id_str(&self, ui_handle: &UiHandle, id_str: &str) -> bool {
        let Some(ui_runtime) = self.ui_runtimes.get(ui_handle) else {
            return false;
        };
        ui_runtime.get_node_id_by_id_str(id_str).is_some()
    }

    pub fn register_ui_event<T: Event + Default>(&mut self, ui_handle: &UiHandle, id_str: &str) {
        let asset_id = ui_handle.asset_id();
        let event_handler = UiNodeEventHandler::new::<T>();

        if let Some(ui_runtime) = self.ui_runtimes.get(&ui_handle) {
            let Some(node_id) = ui_runtime.get_node_id_by_id_str(id_str) else {
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

    pub fn queue_recalculate_layout(&mut self) {
        self.recalc_layout = true;
    }

    pub fn needs_to_recalculate_layout(&self) -> bool {
        self.recalc_layout
    }

    pub fn update_ui_viewport(&mut self, asset_manager: &AssetManager, target_camera: &Camera) {
        let store = asset_manager.get_store();
        let Some(viewport) = target_camera.viewport else {
            return;
        };
        let Some(active_ui_handle) = self.active_ui else {
            return;
        };
        let Some(ui_runtime) = self.ui_runtimes.get_mut(&active_ui_handle) else {
            warn!("ui data not loaded 1: {:?}", active_ui_handle.asset_id());
            return;
        };

        if ui_runtime.update_viewport(&viewport) {
            // viewport did change
            self.queue_recalculate_layout();
        }

        let needs_to_recalc = self.needs_to_recalculate_layout();

        if needs_to_recalc {
            self.recalculate_ui_layout(store, &active_ui_handle);
        }
    }

    fn recalculate_ui_layout(
        &mut self,
        store: &ProcessedAssetStore,
        ui_handle: &UiHandle,
    ) {
        let Some(text_icon_handle) = self.get_text_icon_handle() else {
            return;
        };
        let Some(text_icon_data) = store.icons.get(&text_icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(text_icon_data);

        self.recalculate_ui_layout_impl(&text_measurer, ui_handle, 10.0);
    }

    fn recalculate_ui_layout_impl(
        &mut self,
        text_measurer: &UiTextMeasurer,
        ui_handle: &UiHandle,
        z: f32,
    ) {
        let children = {
            let Some(ui_runtime) = self.ui_runtimes.get_mut(ui_handle) else {
                return;
            };

            self.recalc_layout = false;
            ui_runtime.recalculate_layout(text_measurer, z)
        };

        for (child_ui_handle, child_viewport, child_viewport_z) in children {
            let Some(child_ui_runtime) = self.ui_runtimes.get_mut(&child_ui_handle) else {
                warn!(
                    "child ui data not loaded 1: {:?}",
                    child_ui_handle.asset_id()
                );
                continue;
            };

            child_ui_runtime.update_viewport(&child_viewport);

            self.recalculate_ui_layout_impl(text_measurer, &child_ui_handle, child_viewport_z);
        }
    }

    pub fn generate_new_inputs(&mut self, next_inputs: &mut Vec<UiInputEvent>) {
        let Some((asset_id, node_id)) = self.input_state.get_active_node() else {
            return;
        };
        let ui_handle = UiHandle::new(asset_id);
        let Some(ui_runtime) = self.ui_runtimes.get(&ui_handle) else {
            warn!("ui data not loaded 1: {:?}", asset_id);
            return;
        };
        let (_, config, _, _) = ui_runtime.inner_refs();
        self.input_state
            .generate_new_inputs(config, &node_id, next_inputs);
    }

    pub fn update_ui_input(
        &mut self,
        asset_manager: &AssetManager,
        mouse_position: Option<Vec2>,
        ui_input_events: Vec<UiInputEvent>,
    ) {
        let store = asset_manager.get_store();
        let Some(text_icon_handle) = self.get_text_icon_handle() else {
            return;
        };
        let Some(text_icon_data) = store.icons.get(&text_icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(text_icon_data);
        self.receive_input(&text_measurer, mouse_position, ui_input_events);

        // get any global events
        let mut global_events: Vec<UiGlobalEvent> = self.take_global_events();
        self.ui_global_events.append(&mut global_events);

        // get any node events
        let mut events = self.take_node_events();

        self.ui_node_events.append(&mut events);

        // get cursor icon change
        let new_cursor_icon = self.get_cursor_icon();
        if new_cursor_icon != self.last_cursor_icon {
            self.cursor_icon_change = Some(new_cursor_icon);
        }
    }

    fn receive_input(
        &mut self,
        text_measurer: &UiTextMeasurer,
        mouse_position: Option<Vec2>,
        input_events: Vec<UiInputEvent>,
    ) {
        ui_receive_input(self, text_measurer, mouse_position, input_events);
    }

    pub fn get_textbox_validator(
        &self,
        ui_handle: &UiHandle,
        id_str: &str,
    ) -> Option<ValidationType> {
        let Some(ui_runtime) = self.ui_runtimes.get(ui_handle) else {
            warn!("ui data not loaded 1: {:?}", ui_handle.asset_id());
            return None;
        };
        ui_runtime.get_textbox_validator(id_str)
    }

    pub fn get_textbox_text(&self, ui_handle: &UiHandle, id_str: &str) -> Option<String> {
        let Some(ui_runtime) = self.ui_runtimes.get(ui_handle) else {
            warn!("ui data not loaded 1: {:?}", ui_handle.asset_id());
            return None;
        };
        ui_runtime.get_textbox_text(id_str)
    }

    pub fn set_textbox_text(&mut self, ui_handle: &UiHandle, id_str: &str, val: &str) {
        if let Some(ui_runtime) = self.ui_runtimes.get_mut(ui_handle) {
            ui_runtime.set_textbox_text(id_str, val)
        } else {
            warn!("ui data not loaded 2: {:?}", ui_handle.asset_id());
        }
    }

    pub fn get_node_active_state(&self, ui_id: &UiHandle, node_id: &NodeId) -> NodeActiveState {
        self.input_state
            .get_active_state(&ui_id.asset_id(), node_id)
    }

    pub fn get_node_active_state_from_id(&self, ui_id: &UiHandle, id_str: &str) -> Option<NodeActiveState> {
        let ui_runtime = self.ui_runtimes.get(ui_id)?;
        let node_id = ui_runtime.get_node_id_by_id_str(id_str)?;
        return Some(self.get_node_active_state(ui_id, &node_id));
    }

    pub fn get_text(&self, ui_handle: &UiHandle, id_str: &str) -> Option<String> {
        let Some(ui_runtime) = self.ui_runtimes.get(ui_handle) else {
            warn!("ui data not loaded 3: {:?}", ui_handle.asset_id());
            return None;
        };
        ui_runtime.get_text_by_id_str(id_str)
    }

    pub fn set_text(&mut self, ui_handle: &UiHandle, id_str: &str, val: &str) {
        if let Some(ui_runtime) = self.ui_runtimes.get_mut(ui_handle) {
            ui_runtime.set_text_from_id_str(id_str, val);
            self.queue_recalculate_layout();
        } else {
            warn!("ui data not loaded 4: {:?}", ui_handle.asset_id());
        }
    }

    pub fn set_textbox_password_eye_visible(
        &mut self,
        ui_handle: &UiHandle,
        id_str: &str,
        val: bool,
    ) {
        if let Some(ui_runtime) = self.ui_runtimes.get_mut(ui_handle) {
            ui_runtime.set_textbox_password_eye_visible(id_str, val);
            self.queue_recalculate_layout();
        } else {
            warn!("ui data not loaded 5: {:?}", ui_handle.asset_id());
        }
    }

    pub fn set_node_visible(&mut self, ui_handle: &UiHandle, id_str: &str, val: bool) {
        if let Some(ui_runtime) = self.ui_runtimes.get_mut(ui_handle) {
            ui_runtime.set_node_visible(id_str, val);
            self.queue_recalculate_layout();
        } else {
            warn!("ui data not loaded 6: {:?}", ui_handle.asset_id());
        }
    }

    pub fn get_ui_container_contents(
        &self,
        ui_handle: &UiHandle,
        id_str: &str,
    ) -> Option<UiHandle> {
        let ui_runtime = self.ui_runtimes.get(ui_handle)?;
        ui_runtime.get_ui_container_contents_by_id_str(id_str)
    }

    pub fn set_ui_container_contents(
        &mut self,
        ui_handle: &UiHandle,
        id_str: &str,
        child_ui_handle: &UiHandle,
    ) {
        if let Some(ui_runtime) = self.ui_runtimes.get_mut(ui_handle) {
            ui_runtime.set_ui_container_contents(id_str, child_ui_handle);
            self.queue_recalculate_layout();
        } else {
            warn!("ui data not loaded 7: {:?}", ui_handle.asset_id());
        }
    }

    pub fn clear_ui_container_contents(&mut self, ui_handle: &UiHandle, id_str: &str) {
        if let Some(ui_runtime) = self.ui_runtimes.get_mut(ui_handle) {
            ui_runtime.clear_ui_container_contents(id_str);
            self.queue_recalculate_layout();
        } else {
            warn!("ui data not loaded 8: {:?}", ui_handle.asset_id());
        }
    }

    pub fn get_box_mesh_handle(&self) -> Option<&Handle<CpuMesh>> {
        self.globals.get_box_mesh_handle()
    }

    pub(crate) fn take_global_events(&mut self) -> Vec<UiGlobalEvent> {
        self.input_state.take_global_events()
    }

    pub(crate) fn take_node_events(&mut self) -> Vec<(AssetId, NodeId, UiNodeEvent)> {
        self.input_state.take_node_events()
    }

    pub(crate) fn get_cursor_icon(&self) -> CursorIcon {
        self.input_state.get_cursor_icon()
    }

    pub fn interact_timer_within_seconds(&self, secs: f32) -> bool {
        self.input_state.interact_timer_within_seconds(secs)
    }

    pub fn input_get_select_index(&self) -> Option<usize> {
        self.input_state.select_index
    }

    pub fn input_get_carat_index(&self) -> usize {
        self.input_state.carat_index
    }

    pub fn print_node_tree(&self, handle: &UiHandle) {
        self.print_node(&handle, &NodeId::new(0));
    }

    fn print_node(&self, handle: &UiHandle, node_id: &NodeId) {
        let config = self.ui_runtimes.get(handle).unwrap().ui_config_ref();
        let ui_node = config.get_node(node_id).unwrap();
        info!("{:?} - {:?}", node_id, ui_node);

        for child_id in config.node_children(node_id) {
            self.print_node(handle, child_id);
        }

        if ui_node.widget_kind() == WidgetKind::UiContainer {
            let child_ui_handle = self.ui_runtimes.get(handle).unwrap().get_ui_container_contents(node_id).unwrap();
            self.print_node(&child_ui_handle, &NodeId::new(0));
        }
    }

    pub fn add_copied_node(
        &mut self,
        id_str_map: &mut HashMap<String, NodeId>,
        dest_ui: &UiHandle,
        dest_parent_id: &NodeId,
        src_ui: &UiHandle,
        src_id: &NodeId
    ) -> NodeId {
        let index = self.ui_runtimes.get(dest_ui).unwrap().ui_config_ref().get_node(dest_parent_id).unwrap().widget_panel_ref().unwrap().children.len();
        self.insert_copied_node(index, id_str_map, dest_ui, dest_parent_id, src_ui, src_id)
    }

    // from src to dest, copies entire hierarchy of nodes, recursively
    pub fn insert_copied_node(
        &mut self,
        index: usize,
        id_str_map: &mut HashMap<String, NodeId>,
        dest_ui: &UiHandle,
        dest_parent_id: &NodeId,
        src_ui: &UiHandle,
        src_id: &NodeId
    ) -> NodeId {
        // info!("[{:?} . {:?}] -> [{:?} . {:?}]", src_ui, src_id, dest_ui, dest_parent_id);

        let mut new_copied_node = self.ui_runtimes.get(src_ui).unwrap().ui_config_ref().get_node(src_id).unwrap().clone();
        if let Some(old_style_id) = new_copied_node.style_id() {
            let new_style_id = self.ui_runtimes.get(dest_ui).unwrap().translate_copied_style(src_ui, old_style_id).unwrap();
            new_copied_node.clear_style_id();
            new_copied_node.set_style_id(new_style_id);
            info!("[{:?} . {:?}] added style: [{:?} . {:?}]", src_ui, src_id, dest_ui, new_style_id);
        }
        let old_children_ids_opt: Option<Vec<NodeId>> = if new_copied_node.widget_kind().has_children() {
            let output = Some(new_copied_node.widget.children().unwrap().copied().collect());
            new_copied_node.widget.clear_children();
            info!("[{:?} . {:?}] had old children: {:?}", src_ui, src_id, output);
            output
        } else {
            None
        };

        let dest_runtime = self.ui_runtimes.get_mut(dest_ui).unwrap();
        let Some(mut dest_parent_panel_mut) = dest_runtime.panel_mut(dest_parent_id) else {
            panic!("dest_parent_id is not a panel");
        };

        let new_node_id = dest_parent_panel_mut.insert_node(index, &new_copied_node);

        info!("[ui: {:?} . id: {:?}] -> [ui: {:?}, id: {:?}]", src_ui, src_id, dest_ui, new_node_id);

        if let Some(id_str) = new_copied_node.id_str_opt().as_ref() {
            id_str_map.insert(id_str.clone(), new_node_id);
        }

        // copy children
        if let Some(old_children_ids) = old_children_ids_opt {

            for old_child_id in &old_children_ids {
                self.add_copied_node(id_str_map, dest_ui, &new_node_id, src_ui, old_child_id);
            }

            if let Some(panel_mut) = self.ui_runtimes.get_mut(dest_ui).unwrap().ui_config_mut().panel_mut(&new_node_id) {
                info!("[ui: {:?}, id: {:?}] has new children: {:?}", dest_ui, new_node_id, panel_mut.children);
            } else {
                // it's a button! TODO: handle this case
                warn!("dest_id is not a panel");
            }
        }

        new_node_id
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
