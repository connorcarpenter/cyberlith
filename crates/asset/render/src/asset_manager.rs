use std::collections::HashMap;

use bevy_ecs::{
    change_detection::Mut,
    event::Event,
    system::{ResMut, Res, Resource},
    world::World,
};
use bevy_log::warn;

use asset_id::{AssetId, AssetType};
use clipboard::ClipboardManager;
use input::{CursorIcon, Input};
use render_api::{
    base::CpuSkin,
    base::{CpuMaterial, CpuMesh},
    components::{Camera, RenderLayer, Transform},
    resources::{RenderFrame, Time},
};
use storage::{Handle, Storage};
use ui::{NodeId, TextMeasurer, Ui, UiNodeEvent, UiNodeEventHandler, UiInput, UiGlobalEvent};

use crate::{
    asset_renderer::AssetRenderer, processed_asset_store::ProcessedAssetStore, AnimationData,
    AssetHandle, IconData, MeshData, ModelData, SceneData, SkinData, UiData,
};

#[derive(Resource)]
pub struct AssetManager {
    store: ProcessedAssetStore,

    queued_ui_node_event_handlers: HashMap<AssetHandle<UiData>, Vec<(String, UiNodeEventHandler)>>,
    ui_global_events: Vec<UiGlobalEvent>,
    ui_node_event_handlers: HashMap<(AssetId, NodeId), UiNodeEventHandler>,
    ui_node_events: Vec<(AssetId, NodeId, UiNodeEvent)>,
    cursor_icon_change: Option<CursorIcon>,
    last_cursor_icon: CursorIcon,
    blinkiness: Blinkiness,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            store: ProcessedAssetStore::default(),
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

impl AssetManager {
    pub fn load(
        &mut self,
        asset_data_store: &HashMap<AssetId, Vec<u8>>,
        asset_id: &AssetId,
        asset_type: &AssetType,
    ) {
        self.store.load(asset_data_store, asset_id, asset_type);
    }

    pub fn manual_load_ui(&mut self, asset_id: &AssetId, ui: Ui) {
        self.store.manual_load_ui(asset_id, ui);
    }

    // used as a system
    pub fn sync(
        mut asset_manager: ResMut<Self>,
        mut meshes: ResMut<Storage<CpuMesh>>,
        mut materials: ResMut<Storage<CpuMaterial>>,
        mut skins: ResMut<Storage<CpuSkin>>,
    ) {
        asset_manager.store.sync_meshes(&mut meshes);
        asset_manager.store.sync_icons(&mut meshes);
        asset_manager.store.sync_palettes(&mut materials);
        if let Some(new_uis) = asset_manager.store.sync_uis(&mut meshes, &mut materials) {
            for handle in new_uis {
                if let Some(queued_handlers) =
                    asset_manager.queued_ui_node_event_handlers.remove(&handle)
                {
                    for (id_str, handler) in queued_handlers {
                        let asset_id = handle.asset_id();
                        let ui_store = asset_manager.store.uis.get(&handle).unwrap();
                        let node_id = ui_store
                            .get_ui_ref()
                            .get_node_id_by_id_str(&id_str)
                            .unwrap();
                        asset_manager
                            .ui_node_event_handlers
                            .insert((asset_id, node_id), handler);
                    }
                }
            }
        }

        asset_manager
            .store
            .sync_skins(&meshes, &materials, &mut skins);
        asset_manager
            .store
            .sync_icon_skins(&meshes, &materials, &mut skins);
    }

    // used as a system
    pub fn process_ui_global_events(mut asset_manager: ResMut<AssetManager>, mut clipboard_manager: ResMut<ClipboardManager>) {
        asset_manager.process_ui_global_events_impl(&mut clipboard_manager);
    }

    fn process_ui_global_events_impl(&mut self, clipboard_manager: &mut ClipboardManager) {
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

    // used as a system
    pub fn process_ui_node_events(world: &mut World) {
        world.resource_scope(|world, mut asset_manager: Mut<AssetManager>| {
            asset_manager.process_ui_node_events_impl(world);
        });
    }

    fn process_ui_node_events_impl(&mut self, world: &mut World) {
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

    // used as a system
    pub fn prepare_cursor_change(mut asset_manager: ResMut<AssetManager>) {
        asset_manager.cursor_icon_change = None;
    }

    // used as a system
    pub fn process_cursor_change(mut asset_manager: ResMut<AssetManager>, mut input: ResMut<Input>) {
        let Some(cursor_change) = asset_manager.cursor_icon_change.take() else {
            return;
        };
        if cursor_change != asset_manager.last_cursor_icon {
            asset_manager.last_cursor_icon = cursor_change;
            input.set_cursor_icon(cursor_change);
        }
    }

    // used as a system
    pub fn update_blinkiness(mut asset_manager: ResMut<AssetManager>, time: Res<Time>) {
        let elapsed = time.get_elapsed_ms();
        asset_manager.blinkiness.update(elapsed);
    }

    pub fn register_ui_event<T: Event + Default>(
        &mut self,
        ui_handle: &AssetHandle<UiData>,
        id_str: &str,
    ) {
        let asset_id = ui_handle.asset_id();
        let event_handler = UiNodeEventHandler::new::<T>();

        if let Some(ui_store) = self.store.uis.get(&ui_handle) {
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

    pub fn get_icon_frame_count(&self, handle: &AssetHandle<IconData>) -> usize {
        let data = self.store.icons.get(handle).unwrap();
        data.get_subimage_count()
    }

    pub fn get_icon_max_width(&self, handle: &AssetHandle<IconData>) -> Option<f32> {
        self.store.get_icon_max_width(handle)
    }

    pub fn get_icon_max_height(&self, handle: &AssetHandle<IconData>) -> Option<f32> {
        self.store.get_icon_max_height(handle)
    }

    pub fn get_icon_frame_width(
        &self,
        handle: &AssetHandle<IconData>,
        index: usize,
    ) -> Option<f32> {
        self.store.get_icon_frame_width(handle, index)
    }

    pub fn get_icon_frame_height(
        &self,
        handle: &AssetHandle<IconData>,
        index: usize,
    ) -> Option<f32> {
        self.store.get_icon_frame_height(handle, index)
    }

    pub fn get_animation_duration(&self, handle: &AssetHandle<AnimationData>) -> f32 {
        let data = self.store.animations.get(handle).unwrap();
        data.get_duration()
    }

    pub fn draw_mesh(
        &self,
        render_frame: &mut RenderFrame,
        mesh_handle: &AssetHandle<MeshData>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_mesh(
            render_frame,
            render_layer_opt,
            &self.store,
            mesh_handle,
            mat_handle,
            transform,
        );
    }

    pub fn draw_icon(
        &self,
        render_frame: &mut RenderFrame,
        icon_handle: &AssetHandle<IconData>,
        subimage_index: usize,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_icon(
            render_frame,
            render_layer_opt,
            &self.store,
            icon_handle,
            subimage_index,
            transform,
        );
    }

    pub fn draw_text(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        icon_handle: &AssetHandle<IconData>,
        material_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        text: &str,
    ) {
        AssetRenderer::draw_text(
            render_frame,
            render_layer_opt,
            &self.store,
            icon_handle,
            material_handle,
            transform,
            text,
        );
    }

    pub fn draw_skin(
        &self,
        render_frame: &mut RenderFrame,
        skin_handle: &AssetHandle<SkinData>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_skin(
            render_frame,
            render_layer_opt,
            &self.store,
            skin_handle,
            transform,
        );
    }

    pub fn draw_scene(
        &self,
        render_frame: &mut RenderFrame,
        scene_handle: &AssetHandle<SceneData>,
        parent_transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_scene(
            render_frame,
            render_layer_opt,
            &self.store,
            scene_handle,
            parent_transform,
        );
    }

    pub fn draw_model(
        &self,
        render_frame: &mut RenderFrame,
        model_handle: &AssetHandle<ModelData>,
        parent_transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_model(
            render_frame,
            render_layer_opt,
            &self.store,
            model_handle,
            parent_transform,
        );
    }

    pub fn draw_animated_model(
        &self,
        render_frame: &mut RenderFrame,
        model_handle: &AssetHandle<ModelData>,
        animation_handle: &AssetHandle<AnimationData>,
        parent_transform: &Transform,
        frame_time_ms: f32,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_animated_model(
            render_frame,
            render_layer_opt,
            &self.store,
            model_handle,
            animation_handle,
            parent_transform,
            frame_time_ms,
        );
    }

    pub fn update_ui_viewport(
        &mut self,
        camera: &Camera,
        ui_handle: &AssetHandle<UiData>,
    ) {
        let Some(viewport) = camera.viewport else {
            return;
        };
        let Some(ui_data) = self.store.uis.get_mut(ui_handle) else {
            warn!("ui data not loaded 1: {:?}", ui_handle.asset_id());
            return;
        };

        let ui = ui_data.get_ui_mut();

        ui.update_viewport(&viewport);

        let needs_to_recalc = ui.needs_to_recalculate_layout();

        if needs_to_recalc {
            let icon_handle = ui_data.get_icon_handle();
            self.recalculate_ui_layout(ui_handle, &icon_handle);
        }
    }

    fn recalculate_ui_layout(&mut self, ui_handle: &AssetHandle<UiData>, icon_handle: &AssetHandle<IconData>) {
        let Some(icon_data) = self.store.icons.get(&icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);

        let Some(ui_data) = self.store.uis.get_mut(ui_handle) else {
            return;
        };

        ui_data.get_ui_mut().recalculate_layout(&text_measurer);
    }

    pub fn update_ui_input(
        &mut self,
        ui_input: UiInput,
        ui_handle: &AssetHandle<UiData>,
    ) {
        let Some(ui_data) = self.store.uis.get_mut(ui_handle) else {
            warn!("ui data not loaded 1: {:?}", ui_handle.asset_id());
            return;
        };
        let icon_handle = ui_data.get_icon_handle();
        let Some(icon_data) = self.store.icons.get(&icon_handle) else {
            return;
        };
        let text_measurer = UiTextMeasurer::new(icon_data);
        let ui = ui_data.get_ui_mut();
        ui.receive_input(&text_measurer, ui_input);

        // get any global events
        let mut global_events: Vec<UiGlobalEvent> = ui.take_global_events();
        self.ui_global_events.append(&mut global_events);

        // get any node events
        let mut events: Vec<(AssetId, NodeId, UiNodeEvent)> = ui
            .take_node_events()
            .iter()
            .map(|(node_id, event)| (ui_handle.asset_id(), *node_id, event.clone()))
            .collect();

        self.ui_node_events.append(&mut events);

        // get cursor icon change
        if let Some(cursor_icon) = ui.take_cursor_icon() {
            self.cursor_icon_change = Some(cursor_icon);
        } else {
            self.cursor_icon_change = Some(CursorIcon::Default);
        }
    }

    pub fn draw_ui(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        ui_handle: &AssetHandle<UiData>,
    ) {
        AssetRenderer::draw_ui(render_frame, render_layer_opt, &self.store, &self.blinkiness, ui_handle);
    }
}

pub(crate)  struct UiTextMeasurer<'a> {
    icon_data: &'a IconData,
}

impl<'a> UiTextMeasurer<'a> {
    pub(crate) fn new(icon_data: &'a IconData) -> Self {
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