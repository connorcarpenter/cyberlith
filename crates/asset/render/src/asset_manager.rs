use std::collections::HashMap;

use bevy_ecs::{system::{ResMut, Resource}, event::Event, entity::Entity};
use bevy_log::{info, warn};

use asset_id::{AssetId, AssetType};
use render_api::{
    base::CpuSkin,
    base::{CpuMaterial, CpuMesh},
    components::{RenderLayer, Transform},
    resources::RenderFrame,
};
use storage::{Handle, Storage};
use ui::{NodeId, Ui, WidgetKind};

use crate::{asset_renderer::AssetRenderer, processed_asset_store::ProcessedAssetStore, AnimationData, AssetHandle, IconData, MeshData, ModelData, SceneData, SkinData, UiData};

#[derive(Resource)]
pub struct AssetManager {
    store: ProcessedAssetStore,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            store: ProcessedAssetStore::default(),
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

    pub fn manual_load_ui(
        &mut self,
        asset_id: &AssetId,
        ui: Ui,
    ) {
        self.store.manual_load_ui(asset_id, ui);
    }

    pub fn sync(
        mut asset_manager: ResMut<Self>,
        mut meshes: ResMut<Storage<CpuMesh>>,
        mut materials: ResMut<Storage<CpuMaterial>>,
        mut skins: ResMut<Storage<CpuSkin>>,
    ) {
        asset_manager
            .store
            .sync_meshes(&mut meshes);
        asset_manager
            .store
            .sync_icons(&mut meshes);
        asset_manager
            .store
            .sync_palettes(&mut materials);
        asset_manager
            .store
            .sync_uis(&mut meshes, &mut materials);

        asset_manager
            .store
            .sync_skins(&meshes, &materials, &mut skins);
        asset_manager
            .store
            .sync_icon_skins(&meshes, &materials, &mut skins);
    }

    pub fn register_event<T: Event>(
        &mut self,
        ui_entity: Entity,
        ui_handle: AssetHandle<UiData>,
        id_str: &str,
    ) {
        info!("registering event: {:?}", id_str);
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
        update_button_states(
            ui,
            &Ui::ROOT_NODE_ID,
            mouse_state,
            (0.0, 0.0)
        )
    }

    pub fn draw_ui(
        &self,
        render_frame: &mut RenderFrame,
        render_layer_opt: Option<&RenderLayer>,
        ui_handle: &AssetHandle<UiData>,
    ) {
        AssetRenderer::draw_ui(
            render_frame,
            render_layer_opt,
            &self.store,
            ui_handle,
        );
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
                update_button_states(
                    ui,
                    &child_id,
                    mouse_state,
                    child_position,
                );
            }
        }
        WidgetKind::Button => {
            let Some(button_mut) = ui.store.button_mut(id) else {
                panic!("no button mut for node_id: {:?}", id);
            };
            button_mut.update_state((width, height, child_position.0, child_position.1), mouse_state);
        }
        _ => {}
    }
}