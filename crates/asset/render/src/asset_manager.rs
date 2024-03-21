use std::collections::HashMap;

use bevy_ecs::{
    change_detection::Mut,
    event::Event,
    system::{ResMut, Resource},
    world::World,
};
use bevy_log::warn;

use asset_id::{AssetId, AssetType};
use render_api::{
    base::CpuSkin,
    base::{CpuMaterial, CpuMesh},
    components::{RenderLayer, Transform},
    resources::RenderFrame,
};
use storage::{Handle, Storage};
use ui::{NodeId, Ui, UiEvent, UiEventHandler, WidgetKind};

use crate::{
    asset_renderer::AssetRenderer, processed_asset_store::ProcessedAssetStore, AnimationData,
    AssetHandle, IconData, MeshData, ModelData, SceneData, SkinData, UiData,
};

#[derive(Resource)]
pub struct AssetManager {
    store: ProcessedAssetStore,
    queued_ui_event_handlers: HashMap<AssetHandle<UiData>, Vec<(String, UiEventHandler)>>,
    ui_event_handlers: HashMap<(AssetId, NodeId), UiEventHandler>,
    ui_events: Vec<(AssetId, NodeId, UiEvent)>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            store: ProcessedAssetStore::default(),
            ui_events: Vec::new(),
            ui_event_handlers: HashMap::new(),
            queued_ui_event_handlers: HashMap::new(),
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
                    asset_manager.queued_ui_event_handlers.remove(&handle)
                {
                    for (id_str, handler) in queued_handlers {
                        let asset_id = handle.asset_id();
                        let ui_store = asset_manager.store.uis.get(&handle).unwrap();
                        let node_id = ui_store
                            .get_ui_ref()
                            .get_node_id_by_id_str(&id_str)
                            .unwrap();
                        asset_manager
                            .ui_event_handlers
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
    pub fn process_ui_events(world: &mut World) {
        world.resource_scope(|world, mut asset_manager: Mut<AssetManager>| {
            asset_manager.process_ui_events_impl(world);
        });
    }

    fn process_ui_events_impl(&mut self, world: &mut World) {
        if self.ui_events.is_empty() {
            return;
        }

        let events = std::mem::take(&mut self.ui_events);
        for (asset_id, node_id, event) in events {
            let Some(handler) = self.ui_event_handlers.get(&(asset_id, node_id)) else {
                warn!(
                    "no handler for asset_id: {:?}, node_id: {:?}",
                    asset_id, node_id
                );
                continue;
            };

            handler.handle(world, event);
        }
    }

    pub fn register_ui_event<T: Event + Default>(
        &mut self,
        ui_handle: &AssetHandle<UiData>,
        id_str: &str,
    ) {
        let asset_id = ui_handle.asset_id();
        let event_handler = UiEventHandler::new::<T>();

        if let Some(ui_store) = self.store.uis.get(&ui_handle) {
            let Some(node_id) = ui_store.get_ui_ref().get_node_id_by_id_str(id_str) else {
                panic!("no node_id for id_str: {:?}", id_str);
            };

            self.ui_event_handlers
                .insert((asset_id, node_id), event_handler);
        } else {
            if !self.queued_ui_event_handlers.contains_key(&ui_handle) {
                self.queued_ui_event_handlers
                    .insert(ui_handle.clone(), Vec::new());
            }
            self.queued_ui_event_handlers
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

    pub fn update_ui(
        &mut self,
        render_frame: &RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        mouse_state: (f32, f32, bool),
        ui_handle: &AssetHandle<UiData>,
    ) {
        let Some(ui_data) = self.store.uis.get_mut(ui_handle) else {
            warn!("ui data not loaded 1: {:?}", ui_handle.asset_id());
            return;
        };
        let ui = ui_data.get_ui_mut();

        // update viewport / recalculate layout for ui
        if let Some(viewport) = render_frame.get_camera_viewport(render_layer_opt) {
            ui.update_viewport(&viewport);
            ui.recalculate_layout_if_needed();
        }

        // update button states
        update_button_states(ui, &Ui::ROOT_NODE_ID, mouse_state, (0.0, 0.0));

        // get any events
        let mut events: Vec<(AssetId, NodeId, UiEvent)> = ui
            .take_events()
            .iter()
            .map(|(node_id, event)| (ui_handle.asset_id(), *node_id, event.clone()))
            .collect();

        self.ui_events.append(&mut events);
    }

    pub fn draw_ui(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        ui_handle: &AssetHandle<UiData>,
    ) {
        AssetRenderer::draw_ui(render_frame, render_layer_opt, &self.store, ui_handle);
    }
}

fn update_button_states(
    ui: &mut Ui,
    id: &NodeId,
    mouse_state: (f32, f32, bool),
    parent_position: (f32, f32),
) {
    let Some(node) = ui.store.get_node(&id) else {
        warn!("no panel for id: {:?}", id);
        return;
    };

    if !node.visible {
        return;
    }

    let Some((width, height, child_offset_x, child_offset_y)) = ui.cache.bounds(id) else {
        warn!("no bounds for id: {:?}", id);
        return;
    };

    let child_position = (
        parent_position.0 + child_offset_x,
        parent_position.1 + child_offset_y,
    );

    match node.widget_kind() {
        WidgetKind::Panel => {
            let Some(panel_ref) = ui.store.panel_ref(id) else {
                panic!("no panel ref for node_id: {:?}", id);
            };

            // update children
            let child_ids = panel_ref.children.clone();
            for child_id in child_ids {
                update_button_states(ui, &child_id, mouse_state, child_position);
            }
        }
        WidgetKind::Button => {
            let Some(button_mut) = ui.store.button_mut(id) else {
                panic!("no button mut for node_id: {:?}", id);
            };
            let did_click = button_mut.update_state(
                (width, height, child_position.0, child_position.1),
                mouse_state,
            );
            if did_click {
                ui.emit_event(id, UiEvent::Clicked);
            }
        }
        _ => {}
    }
}
