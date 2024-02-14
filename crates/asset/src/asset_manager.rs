
use bevy_ecs::system::{ResMut, Resource};
use asset_io::AssetId;

use render_api::{
    base::CpuSkin,
    base::{CpuMaterial, CpuMesh},
    components::{RenderLayer, Transform},
    resources::RenderFrame,
};
use storage::{Storage, Handle};

use crate::{
    AnimationData, AssetHandle,
    IconData, MeshFile, ModelData, SceneData, SkinData, asset_renderer::AssetRenderer, asset_store::AssetStore
};

#[derive(Resource)]
pub struct AssetManager {
    store: AssetStore,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            store: AssetStore::default(),
        }
    }
}

impl AssetManager {
    pub fn load<T: From<AssetHandle>>(&mut self, asset_id: &AssetId) -> T {
        self.store.load::<T>(asset_id)
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
        asset_manager.store.sync_skins(&meshes, &materials, &mut skins);
        asset_manager.store.sync_icon_skins(&meshes, &materials, &mut skins);
    }

    pub fn get_icon_subimage_count(&self, handle: &Handle<IconData>) -> usize {
        let data = self.store.icons.get(handle).unwrap();
        data.get_subimage_count()
    }

    pub fn get_animation_duration(&self, handle: &Handle<AnimationData>) -> f32 {
        let data = self.store.animations.get(handle).unwrap();
        data.get_duration()
    }

    pub fn draw_mesh(
        &self,
        render_frame: &mut RenderFrame,
        mesh_handle: &Handle<MeshFile>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_mesh(&self.store, render_frame, mesh_handle, mat_handle, transform, render_layer_opt);
    }

    pub fn draw_icon(
        &self,
        render_frame: &mut RenderFrame,
        icon_handle: &Handle<IconData>,
        subimage_index: usize,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_icon(&self.store, render_frame, icon_handle, subimage_index, transform, render_layer_opt);
    }

    pub fn draw_skin(
        &self,
        render_frame: &mut RenderFrame,
        skin_handle: &Handle<SkinData>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_skin(&self.store, render_frame, skin_handle, transform, render_layer_opt);
    }

    pub fn draw_scene(
        &self,
        render_frame: &mut RenderFrame,
        scene_handle: &Handle<SceneData>,
        parent_transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_scene(&self.store, render_frame, scene_handle, parent_transform, render_layer_opt);
    }

    pub fn draw_model(
        &self,
        render_frame: &mut RenderFrame,
        model_handle: &Handle<ModelData>,
        parent_transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        AssetRenderer::draw_model(&self.store, render_frame, model_handle, parent_transform, render_layer_opt);
    }

    pub fn draw_animated_model(
        &self,
        render_frame: &mut RenderFrame,
        model_handle: &Handle<ModelData>,
        animation_handle: &Handle<AnimationData>,
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
