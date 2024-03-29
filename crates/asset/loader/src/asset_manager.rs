use std::collections::HashMap;

use bevy_ecs::system::{ResMut, Resource};

use asset_id::{AssetId, AssetType};
use render_api::{
    base::{CpuMaterial, CpuMesh},
    base::CpuSkin,
};
use storage::Storage;

use crate::{
    AnimationData, AssetHandle,
    IconData, processed_asset_store::ProcessedAssetStore,
};

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

    // used as a system
    pub(crate) fn sync(
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

    pub fn get_store(&self) -> &ProcessedAssetStore {
        &self.store
    }

    pub fn get_store_mut(&mut self) -> &mut ProcessedAssetStore {
        &mut self.store
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
}