use std::collections::HashMap;

use bevy_log::warn;

use asset_id::AssetId;
use render_api::base::{CpuMaterial, CpuMesh, CpuSkin};
use storage::{Handle, Storage};

use crate::{
    asset_handle::AssetHandleImpl, AnimationData, AssetHandle, IconData, MeshFile, ModelData,
    PaletteData, SceneData, SkeletonData, SkinData,
};

pub(crate) struct AssetStore {
    pub(crate) meshes: Storage<MeshFile>,
    pub(crate) skeletons: Storage<SkeletonData>,
    pub(crate) palettes: Storage<PaletteData>,
    pub(crate) animations: Storage<AnimationData>,
    pub(crate) icons: Storage<IconData>,
    pub(crate) skins: Storage<SkinData>,
    pub(crate) models: Storage<ModelData>,
    pub(crate) scenes: Storage<SceneData>,

    // mesh file name, skin handle
    queued_meshes: Vec<Handle<MeshFile>>,
    queued_palettes: Vec<Handle<PaletteData>>,
    queued_icons: Vec<Handle<IconData>>,
    icons_waiting_on_palettes: HashMap<Handle<PaletteData>, Vec<Handle<IconData>>>,
    skins_waiting_on_palettes: HashMap<Handle<PaletteData>, Vec<Handle<SkinData>>>,
    skins_waiting_on_meshes: HashMap<Handle<MeshFile>, Vec<Handle<SkinData>>>,
    ready_icons: Vec<Handle<IconData>>,
    ready_skins: Vec<Handle<SkinData>>,
}

impl Default for AssetStore {
    fn default() -> Self {
        Self {
            meshes: Storage::default(),
            skeletons: Storage::default(),
            palettes: Storage::default(),
            animations: Storage::default(),
            icons: Storage::default(),
            skins: Storage::default(),
            models: Storage::default(),
            scenes: Storage::default(),

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

impl AssetStore {
    pub(crate) fn load<T: From<AssetHandle>>(&mut self, asset_id: &AssetId) -> T {
        todo!();
        //
        // let file_ext: String = todo!(); // need to get extension from asset id
        //
        // let mut dependencies = Vec::new();
        //
        // let asset_handle = match file_ext.as_str() {
        //     "mesh" => {
        //         let existed = self.meshes.has(*asset_id);
        //         let handle = self.meshes.add(*asset_id);
        //         if !existed {
        //             self.queued_meshes.push(handle.clone());
        //         }
        //         let asset_handle: AssetHandle = handle.into();
        //         asset_handle.into()
        //     }
        //     "skeleton" => {
        //         let asset_handle: AssetHandle = self.skeletons.add(*asset_id).into();
        //         asset_handle.into()
        //     }
        //     "palette" => {
        //         let existed = self.palettes.has(*asset_id);
        //         let handle = self.palettes.add(*asset_id);
        //         if !existed {
        //             self.queued_palettes.push(handle.clone());
        //         }
        //         let asset_handle: AssetHandle = handle.into();
        //         asset_handle.into()
        //     }
        //     "animation" => {
        //         let existed = self.animations.has(*asset_id);
        //         let handle = self.animations.add(*asset_id);
        //         if !existed {
        //             let data = self.animations.get(&handle).unwrap();
        //             data.load_dependencies(handle, &mut dependencies);
        //         }
        //         let asset_handle: AssetHandle = handle.into();
        //         asset_handle.into()
        //     }
        //     "icon" => {
        //         let existed = self.icons.has(*asset_id);
        //         let handle = self.icons.add(*asset_id);
        //         if !existed {
        //             self.queued_icons.push(handle.clone());
        //             let data = self.icons.get(&handle).unwrap();
        //             data.load_dependencies(handle, &mut dependencies);
        //         }
        //         let asset_handle: AssetHandle = handle.into();
        //         asset_handle.into()
        //     }
        //     "skin" => {
        //         let existed = self.skins.has(*asset_id);
        //         let handle = self.skins.add(*asset_id);
        //         if !existed {
        //             let data = self.skins.get(&handle).unwrap();
        //             data.load_dependencies(handle, &mut dependencies);
        //         }
        //         let asset_handle: AssetHandle = handle.into();
        //         asset_handle.into()
        //     }
        //     "model" => {
        //         let existed = self.models.has(*asset_id);
        //         let handle = self.models.add(*asset_id);
        //         if !existed {
        //             let data = self.models.get(&handle).unwrap();
        //             data.load_dependencies(handle, &mut dependencies);
        //         }
        //         let asset_handle: AssetHandle = handle.into();
        //         asset_handle.into()
        //     }
        //     "scene" => {
        //         let existed = self.scenes.has(*asset_id);
        //         let handle = self.scenes.add(*asset_id);
        //         if !existed {
        //             let data = self.scenes.get(&handle).unwrap();
        //             data.load_dependencies(handle, &mut dependencies);
        //         }
        //         let asset_handle: AssetHandle = handle.into();
        //         asset_handle.into()
        //     }
        //     _ => panic!("Unknown file extension: {}", file_ext),
        // };
        //
        // if !dependencies.is_empty() {
        //     for (principal_handle, dependency_id) in dependencies {
        //         let dependency_handle = self.load(&dependency_id);
        //         self.finish_dependency(principal_handle, dependency_id, dependency_handle);
        //     }
        // }
        //
        // asset_handle
    }

    fn finish_dependency(
        &mut self,
        principal_handle: AssetHandle,
        dependency_id: AssetId,
        dependency_handle: AssetHandle,
    ) {
        match principal_handle.to_impl() {
            AssetHandleImpl::Mesh(_)
            | AssetHandleImpl::Skeleton(_)
            | AssetHandleImpl::Palette(_) => {
                panic!("unexpected dependency for this type of asset")
            }
            AssetHandleImpl::Animation(principal_handle) => {
                let data = self.animations.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_id, dependency_handle);
            }
            AssetHandleImpl::Icon(principal_handle) => {
                let data = self.icons.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_id, dependency_handle);
                if data.has_all_dependencies() {
                    let palette_handle = data.get_palette_file_handle().unwrap().clone();

                    if !self.palette_has_cpu_materials(&palette_handle) {
                        if !self.icons_waiting_on_palettes.contains_key(&palette_handle) {
                            self.icons_waiting_on_palettes
                                .insert(palette_handle.clone(), Vec::new());
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
            AssetHandleImpl::Skin(principal_handle) => {
                let data = self.skins.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_id, dependency_handle);
                if data.has_all_dependencies() {
                    let palette_handle = data.get_palette_file_handle().unwrap().clone();
                    let mesh_handle = data.get_mesh_file_handle().unwrap().clone();

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
            AssetHandleImpl::Model(principal_handle) => {
                let data = self.models.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_id, dependency_handle);

                if data.all_dependencies_loaded() {
                    let skeleton_handle = data.get_skeleton_handle();
                    let skeleton_data = self.skeletons.get(&skeleton_handle).unwrap();
                    data.compute_components(skeleton_data);
                }
            }
            AssetHandleImpl::Scene(principal_handle) => {
                let data = self.scenes.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_id, dependency_handle);
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
                warn!("skin data {} not loaded, re-queuing", skin_handle.id);
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
                warn!("icon data {} not loaded, re-queuing", icon_handle.id);
                self.ready_icons.push(icon_handle);
            }
        }
    }

    fn palette_has_cpu_materials(&self, palette_handle: &Handle<PaletteData>) -> bool {
        let data = self.palettes.get(palette_handle).unwrap();
        data.has_cpu_materials()
    }

    fn mesh_file_has_cpu_mesh(&self, mesh_handle: &Handle<MeshFile>) -> bool {
        let data = self.meshes.get(mesh_handle).unwrap();
        data.has_cpu_mesh()
    }

    fn icon_is_ready(&self, icon_handle: &Handle<IconData>) -> bool {
        let data = self.icons.get(icon_handle).unwrap();

        let palette_handle = data.get_palette_file_handle().unwrap();

        if data.has_all_cpu_meshes() && self.palette_has_cpu_materials(palette_handle) {
            return true;
        }
        return false;
    }

    fn skin_is_ready(&self, skin_handle: &Handle<SkinData>) -> bool {
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