use std::collections::HashMap;

use bevy_log::{info, warn};

use asset_id::{AssetId, AssetType};
use render_api::base::{CpuMaterial, CpuMesh, CpuSkin};
use storage::Storage;

use crate::{
    asset_storage::AssetStorage, AnimationData, AssetHandle, IconData, MeshData, ModelData,
    PaletteData, SceneData, SkeletonData, SkinData, TypedAssetId,
};

pub(crate) struct ProcessedAssetStore {
    pub(crate) meshes: AssetStorage<MeshData>,
    pub(crate) skeletons: AssetStorage<SkeletonData>,
    pub(crate) palettes: AssetStorage<PaletteData>,
    pub(crate) animations: AssetStorage<AnimationData>,
    pub(crate) icons: AssetStorage<IconData>,
    pub(crate) skins: AssetStorage<SkinData>,
    pub(crate) models: AssetStorage<ModelData>,
    pub(crate) scenes: AssetStorage<SceneData>,

    // mesh file name, skin handle
    queued_meshes: Vec<AssetHandle<MeshData>>,
    queued_palettes: Vec<AssetHandle<PaletteData>>,
    queued_icons: Vec<AssetHandle<IconData>>,
    icons_waiting_on_palettes: HashMap<AssetHandle<PaletteData>, Vec<AssetHandle<IconData>>>,
    skins_waiting_on_palettes: HashMap<AssetHandle<PaletteData>, Vec<AssetHandle<SkinData>>>,
    skins_waiting_on_meshes: HashMap<AssetHandle<MeshData>, Vec<AssetHandle<SkinData>>>,
    ready_icons: Vec<AssetHandle<IconData>>,
    ready_skins: Vec<AssetHandle<SkinData>>,
}

impl Default for ProcessedAssetStore {
    fn default() -> Self {
        Self {
            meshes: AssetStorage::default(),
            skeletons: AssetStorage::default(),
            palettes: AssetStorage::default(),
            animations: AssetStorage::default(),
            icons: AssetStorage::default(),
            skins: AssetStorage::default(),
            models: AssetStorage::default(),
            scenes: AssetStorage::default(),

            queued_meshes: Vec::new(),
            queued_palettes: Vec::new(),
            queued_icons: Vec::new(),
            icons_waiting_on_palettes: HashMap::new(),
            skins_waiting_on_palettes: HashMap::new(),
            skins_waiting_on_meshes: HashMap::new(),
            ready_icons: Vec::new(),
            ready_skins: Vec::new(),
        }
    }
}

impl ProcessedAssetStore {
    pub(crate) fn get_icon_frame_width(
        &self,
        handle: &AssetHandle<IconData>,
        frame_index: usize,
    ) -> Option<f32> {
        let data = self.icons.get(handle)?;
        data.get_frame_width(frame_index)
    }

    pub(crate) fn get_icon_frame_height(
        &self,
        handle: &AssetHandle<IconData>,
        frame_index: usize,
    ) -> Option<f32> {
        let data = self.icons.get(handle)?;
        data.get_frame_height(frame_index)
    }

    pub(crate) fn load(
        &mut self,
        asset_data_store: &HashMap<AssetId, Vec<u8>>,
        asset_id: &AssetId,
        asset_type: &AssetType,
    ) {
        let mut dependencies: Vec<(TypedAssetId, TypedAssetId)> = Vec::new();

        match asset_type {
            AssetType::Mesh => {
                let handle = AssetHandle::<MeshData>::new(*asset_id);
                if !self.meshes.has(&handle) {
                    let bytes = asset_data_store.get(asset_id).unwrap();
                    let mesh_data = MeshData::from_bytes(bytes);
                    self.meshes.insert(handle, mesh_data);
                    self.queued_meshes.push(handle);
                }
            }
            AssetType::Skeleton => {
                let handle = AssetHandle::<SkeletonData>::new(*asset_id);
                if !self.skeletons.has(&handle) {
                    let bytes = asset_data_store.get(asset_id).unwrap();
                    let skeleton_data = SkeletonData::from_bytes(bytes);
                    self.skeletons.insert(handle, skeleton_data);
                }
            }
            AssetType::Palette => {
                let handle = AssetHandle::<PaletteData>::new(*asset_id);
                if !self.palettes.has(&handle) {
                    let bytes = asset_data_store.get(asset_id).unwrap();
                    let palette_data = PaletteData::from_bytes(bytes);
                    self.palettes.insert(handle, palette_data);
                    self.queued_palettes.push(handle);
                }
            }
            AssetType::Animation => {
                let handle = AssetHandle::<AnimationData>::new(*asset_id);
                if !self.animations.has(&handle) {
                    let bytes = asset_data_store.get(asset_id).unwrap();
                    let animation_data = AnimationData::from_bytes(bytes);
                    self.animations.insert(handle, animation_data);
                    let animation_data = self.animations.get(&handle).unwrap();
                    animation_data.load_dependencies(handle, &mut dependencies);
                }
            }
            AssetType::Icon => {
                let handle = AssetHandle::<IconData>::new(*asset_id);
                if !self.icons.has(&handle) {
                    let bytes = asset_data_store.get(asset_id).unwrap();
                    let icon_data = IconData::from_bytes(bytes);
                    self.icons.insert(handle, icon_data);
                    self.queued_icons.push(handle);
                    let icon_data = self.icons.get(&handle).unwrap();
                    icon_data.load_dependencies(handle, &mut dependencies);
                }
            }
            AssetType::Skin => {
                let handle = AssetHandle::<SkinData>::new(*asset_id);
                if !self.skins.has(&handle) {
                    info!("loading skin {:?}", handle.asset_id());
                    let bytes = asset_data_store.get(asset_id).unwrap();
                    let skin_data = SkinData::from_bytes(bytes);
                    self.skins.insert(handle, skin_data);
                    let skin_data = self.skins.get(&handle).unwrap();
                    skin_data.load_dependencies(handle, &mut dependencies);
                }
            }
            AssetType::Model => {
                let handle = AssetHandle::<ModelData>::new(*asset_id);
                if !self.models.has(&handle) {
                    let bytes = asset_data_store.get(asset_id).unwrap();
                    let model_data = ModelData::from_bytes(bytes);
                    self.models.insert(handle, model_data);
                    let model_data = self.models.get(&handle).unwrap();
                    model_data.load_dependencies(handle, &mut dependencies);
                }
            }
            AssetType::Scene => {
                let handle = AssetHandle::<SceneData>::new(*asset_id);
                if !self.scenes.has(&handle) {
                    let bytes = asset_data_store.get(asset_id).unwrap();
                    let scene_data = SceneData::from_bytes(bytes);
                    self.scenes.insert(handle, scene_data);
                    let scene_data = self.scenes.get(&handle).unwrap();
                    scene_data.load_dependencies(handle, &mut dependencies);
                }
            }
        };

        if !dependencies.is_empty() {
            for (principal_handle, dependency_handle) in dependencies {
                let dependency_id = dependency_handle.get_id();
                let dependency_type = dependency_handle.get_type();
                self.load(asset_data_store, &dependency_id, &dependency_type);
                self.finish_dependency(principal_handle, dependency_handle);
            }
        }
    }

    fn finish_dependency(
        &mut self,
        principal_typed_id: TypedAssetId,
        dependency_typed_id: TypedAssetId,
    ) {
        match principal_typed_id {
            TypedAssetId::Mesh(_) | TypedAssetId::Skeleton(_) | TypedAssetId::Palette(_) => {
                panic!("unexpected dependency for this type of asset")
            }
            TypedAssetId::Animation(principal_id) => {
                let principal_handle = AssetHandle::<AnimationData>::new(principal_id);
                let principal_data = self.animations.get_mut(&principal_handle).unwrap();
                principal_data.finish_dependency(dependency_typed_id);
            }
            TypedAssetId::Icon(principal_id) => {
                let principal_handle = AssetHandle::<IconData>::new(principal_id);
                let principal_data = self.icons.get_mut(&principal_handle).unwrap();
                principal_data.finish_dependency(dependency_typed_id);
                if principal_data.has_all_dependencies() {
                    let palette_handle = *principal_data.get_palette_file_handle().unwrap();

                    if !self.palette_has_cpu_materials(&palette_handle) {
                        if !self.icons_waiting_on_palettes.contains_key(&palette_handle) {
                            self.icons_waiting_on_palettes
                                .insert(palette_handle, Vec::new());
                        }
                        let icon_list = self
                            .icons_waiting_on_palettes
                            .get_mut(&palette_handle)
                            .unwrap();
                        icon_list.push(principal_handle);
                    }

                    if self.icon_is_ready(&principal_handle) {
                        self.ready_icons.push(principal_handle);
                    }
                }
            }
            TypedAssetId::Skin(principal_id) => {
                let principal_handle = AssetHandle::<SkinData>::new(principal_id);
                let principal_data = self.skins.get_mut(&principal_handle).unwrap();
                principal_data.finish_dependency(dependency_typed_id);
                if principal_data.has_all_dependencies() {
                    let palette_handle = *principal_data.get_palette_file_handle().unwrap();
                    let mesh_handle = *principal_data.get_mesh_file_handle().unwrap();

                    if !self.palette_has_cpu_materials(&palette_handle) {
                        if !self.skins_waiting_on_palettes.contains_key(&palette_handle) {
                            self.skins_waiting_on_palettes
                                .insert(palette_handle.clone(), Vec::new());
                        }
                        let skin_list = self
                            .skins_waiting_on_palettes
                            .get_mut(&palette_handle)
                            .unwrap();
                        skin_list.push(principal_handle);
                    }
                    if !self.mesh_file_has_cpu_mesh(&mesh_handle) {
                        if !self.skins_waiting_on_meshes.contains_key(&mesh_handle) {
                            self.skins_waiting_on_meshes
                                .insert(mesh_handle.clone(), Vec::new());
                        }
                        let skin_list = self.skins_waiting_on_meshes.get_mut(&mesh_handle).unwrap();
                        skin_list.push(principal_handle);
                    }

                    if self.skin_is_ready(&principal_handle) {
                        self.ready_skins.push(principal_handle);
                    }
                }
            }
            TypedAssetId::Model(principal_id) => {
                let principal_handle = AssetHandle::<ModelData>::new(principal_id);
                let principal_data = self.models.get_mut(&principal_handle).unwrap();
                principal_data.finish_dependency(dependency_typed_id);

                if principal_data.all_dependencies_loaded() {
                    let skeleton_handle = principal_data.get_skeleton_handle();
                    let skeleton_data = self.skeletons.get(&skeleton_handle).unwrap();
                    principal_data.compute_components(skeleton_data);
                }
            }
            TypedAssetId::Scene(principal_id) => {
                let prinicipal_handle = AssetHandle::<SceneData>::new(principal_id);
                let principal_data = self.scenes.get_mut(&prinicipal_handle).unwrap();
                principal_data.finish_dependency(dependency_typed_id);
            }
        }
    }

    pub(crate) fn sync_meshes(&mut self, meshes: &mut Storage<CpuMesh>) {
        if self.queued_meshes.is_empty() {
            return;
        }

        let mesh_handles = std::mem::take(&mut self.queued_meshes);

        for mesh_handle in &mesh_handles {
            let mesh_file = self.meshes.get_mut(&mesh_handle).unwrap();
            mesh_file.load_cpu_mesh(meshes);
        }

        for mesh_handle in &mesh_handles {
            if let Some(skin_list) = self.skins_waiting_on_meshes.remove(mesh_handle) {
                for skin_handle in skin_list {
                    if self.skin_is_ready(&skin_handle) {
                        self.ready_skins.push(skin_handle);
                    }
                }
            }
        }
    }

    pub(crate) fn sync_palettes(&mut self, materials: &mut Storage<CpuMaterial>) {
        if self.queued_palettes.is_empty() {
            return;
        }

        let palette_handles = std::mem::take(&mut self.queued_palettes);

        for palette_handle in &palette_handles {
            let palette_data = self.palettes.get_mut(palette_handle).unwrap();
            palette_data.load_cpu_materials(materials);
        }

        for palette_handle in &palette_handles {
            if let Some(skin_list) = self.skins_waiting_on_palettes.remove(palette_handle) {
                for skin_handle in skin_list {
                    if self.skin_is_ready(&skin_handle) {
                        self.ready_skins.push(skin_handle);
                    }
                }
            }

            if let Some(icon_list) = self.icons_waiting_on_palettes.remove(palette_handle) {
                for icon_handle in icon_list {
                    if self.icon_is_ready(&icon_handle) {
                        self.ready_icons.push(icon_handle);
                    }
                }
            }
        }
    }

    pub(crate) fn sync_skins(
        &mut self,
        meshes: &Storage<CpuMesh>,
        materials: &Storage<CpuMaterial>,
        skins: &mut Storage<CpuSkin>,
    ) {
        if self.ready_skins.is_empty() {
            return;
        }

        for skin_handle in std::mem::take(&mut self.ready_skins) {
            let skin_data = self.skins.get_mut(&skin_handle).unwrap();

            let mesh_file_handle = skin_data.get_mesh_file_handle().unwrap();
            let mesh_file_data = self.meshes.get(mesh_file_handle).unwrap();
            let mesh_handle = mesh_file_data.get_cpu_mesh_handle().unwrap();
            let mesh_data = meshes.get(mesh_handle).unwrap();

            let palette_handle = skin_data.get_palette_file_handle().unwrap();
            let palette_data = self.palettes.get(palette_handle).unwrap();

            if skin_data.load_cpu_skin(materials, skins, mesh_data, palette_data) {
                // success!
            } else {
                warn!(
                    "skin data {:?} not loaded, re-queuing",
                    skin_handle.asset_id()
                );
                self.ready_skins.push(skin_handle);
            }
        }
    }

    pub(crate) fn sync_icons(&mut self, meshes: &mut Storage<CpuMesh>) {
        if self.queued_icons.is_empty() {
            return;
        }

        let icon_handles = std::mem::take(&mut self.queued_icons);

        for icon_handle in &icon_handles {
            let icon_data = self.icons.get_mut(&icon_handle).unwrap();
            icon_data.load_cpu_meshes(meshes);
        }

        for icon_handle in &icon_handles {
            if self.icon_is_ready(icon_handle) {
                self.ready_icons.push(*icon_handle);
            }
        }
    }

    pub(crate) fn sync_icon_skins(
        &mut self,
        meshes: &Storage<CpuMesh>,
        materials: &Storage<CpuMaterial>,
        skins: &mut Storage<CpuSkin>,
    ) {
        if self.ready_icons.is_empty() {
            return;
        }

        for icon_handle in std::mem::take(&mut self.ready_icons) {
            let icon_data = self.icons.get_mut(&icon_handle).unwrap();

            let palette_handle = icon_data.get_palette_file_handle().unwrap();
            let palette_data = self.palettes.get(palette_handle).unwrap();

            if icon_data.load_cpu_skins(meshes, materials, skins, palette_data) {
                // success!
            } else {
                warn!(
                    "icon data {:?} not loaded, re-queuing",
                    icon_handle.asset_id()
                );
                self.ready_icons.push(icon_handle);
            }
        }
    }

    fn palette_has_cpu_materials(&self, palette_handle: &AssetHandle<PaletteData>) -> bool {
        let data = self.palettes.get(palette_handle).unwrap();
        data.has_cpu_materials()
    }

    fn mesh_file_has_cpu_mesh(&self, mesh_handle: &AssetHandle<MeshData>) -> bool {
        let data = self.meshes.get(mesh_handle).unwrap();
        data.has_cpu_mesh()
    }

    fn icon_is_ready(&self, icon_handle: &AssetHandle<IconData>) -> bool {
        let data = self.icons.get(icon_handle).unwrap();

        let palette_handle = data.get_palette_file_handle().unwrap();

        if data.has_all_cpu_meshes() && self.palette_has_cpu_materials(palette_handle) {
            return true;
        }
        return false;
    }

    fn skin_is_ready(&self, skin_handle: &AssetHandle<SkinData>) -> bool {
        let data = self.skins.get(skin_handle).unwrap();

        let mesh_handle = data.get_mesh_file_handle().unwrap();
        let palette_handle = data.get_palette_file_handle().unwrap();

        if self.mesh_file_has_cpu_mesh(mesh_handle)
            && self.palette_has_cpu_materials(palette_handle)
        {
            return true;
        }
        return false;
    }
}
