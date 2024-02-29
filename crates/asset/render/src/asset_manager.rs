use std::collections::HashMap;

use bevy_ecs::system::{ResMut, Resource};

use asset_id::{AssetId, AssetType};
use math::Vec3;
use render_api::{
    base::CpuSkin,
    base::{CpuMaterial, CpuMesh},
    components::{RenderLayer, Transform},
    resources::RenderFrame,
};
use storage::{Handle, Storage};

use crate::{asset_renderer::AssetRenderer, processed_asset_store::ProcessedAssetStore, AnimationData, AssetHandle, IconData, MeshData, ModelData, SceneData, SkinData, TextStyle};

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

    pub fn sync(
        mut asset_manager: ResMut<Self>,
        mut meshes: ResMut<Storage<CpuMesh>>,
        mut materials: ResMut<Storage<CpuMaterial>>,
        mut skins: ResMut<Storage<CpuSkin>>,
    ) {
        asset_manager.store.sync_meshes(&mut meshes);
        asset_manager.store.sync_icons(&mut meshes);
        asset_manager.store.sync_palettes(&mut materials);
        asset_manager
            .store
            .sync_skins(&meshes, &materials, &mut skins);
        asset_manager
            .store
            .sync_icon_skins(&meshes, &materials, &mut skins);
    }

    pub fn get_icon_frame_count(&self, handle: &AssetHandle<IconData>) -> usize {
        let data = self.store.icons.get(handle).unwrap();
        data.get_subimage_count()
    }

    pub fn get_icon_frame_width(&self, handle: &AssetHandle<IconData>, index: usize) -> Option<f32> {
        self.store.get_icon_frame_width(handle, index)
    }

    pub fn get_icon_frame_height(&self, handle: &AssetHandle<IconData>, index: usize) -> Option<f32> {
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
            &self.store,
            render_frame,
            mesh_handle,
            mat_handle,
            transform,
            render_layer_opt,
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
            &self.store,
            render_frame,
            icon_handle,
            subimage_index,
            transform,
            render_layer_opt,
        );
    }

    pub fn draw_text(
        &self,
        render_frame: &mut RenderFrame,
        icon_handle: &AssetHandle<IconData>,
        style: &TextStyle,
        position: &Vec3,
        render_layer_opt: Option<&RenderLayer>,
        text: &str,
    ) {
        AssetRenderer::draw_text(
            &self.store,
            render_frame,
            icon_handle,
            style,
            position,
            render_layer_opt,
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
            &self.store,
            render_frame,
            skin_handle,
            transform,
            render_layer_opt,
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
            &self.store,
            render_frame,
            scene_handle,
            parent_transform,
            render_layer_opt,
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
            &self.store,
            render_frame,
            model_handle,
            parent_transform,
            render_layer_opt,
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
            &self.store,
            render_frame,
            model_handle,
            animation_handle,
            parent_transform,
            frame_time_ms,
            render_layer_opt,
        );
    }
}
