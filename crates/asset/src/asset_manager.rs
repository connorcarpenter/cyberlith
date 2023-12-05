use std::collections::HashMap;
use bevy_ecs::system::{ResMut, Resource};
use bevy_log::warn;

use render_api::{Assets, Handle, base::{CpuMaterial, CpuMesh}, components::{RenderLayer, Transform}, resources::RenderFrame};
use render_api::base::CpuSkin;

use crate::{asset_handle::AssetHandleImpl, AnimationData, AssetHandle, IconData, MeshFile, ModelData, PaletteData, SceneData, SkeletonData, SkinData};

#[derive(Resource)]
pub struct AssetManager {
    meshes: Assets<MeshFile>,
    skeletons: Assets<SkeletonData>,
    palettes: Assets<PaletteData>,
    animations: Assets<AnimationData>,
    icons: Assets<IconData>,
    skins: Assets<SkinData>,
    models: Assets<ModelData>,
    scenes: Assets<SceneData>,

    // mesh file name, skin handle
    queued_meshes: Vec<Handle<MeshFile>>,
    queued_palettes: Vec<Handle<PaletteData>>,
    skins_waiting_on_palettes: HashMap<Handle<PaletteData>, Handle<SkinData>>,
    ready_skins: Vec<(Handle<SkinData>, Handle<PaletteData>)>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            meshes: Assets::default(),
            skeletons: Assets::default(),
            palettes: Assets::default(),
            animations: Assets::default(),
            icons: Assets::default(),
            skins: Assets::default(),
            models: Assets::default(),
            scenes: Assets::default(),

            queued_meshes: Vec::new(),
            queued_palettes: Vec::new(),
            skins_waiting_on_palettes: HashMap::new(),
            ready_skins: Vec::new(),
        }
    }
}

impl AssetManager {
    pub fn load<T: From<AssetHandle>>(&mut self, path: &str) -> T {
        let file_ext = path.split('.').last().unwrap();
        let path_string = path.to_string();

        let mut dependencies = Vec::new();

        let asset_handle = match file_ext {
            "mesh" => {
                let existed = self.meshes.has(path_string.clone());
                let handle = self.meshes.add(path_string);
                if !existed {
                    self.queued_meshes.push(handle.clone());
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            },
            "skel" => {
                let asset_handle: AssetHandle = self.skeletons.add(path_string).into();
                asset_handle.into()
            },
            "palette" => {
                let existed = self.palettes.has(path_string.clone());
                let handle = self.palettes.add(path_string);
                if !existed {
                    self.queued_palettes.push(handle.clone());
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            },
            "anim" => {
                let existed = self.animations.has(path_string.clone());
                let handle = self.animations.add(path_string);
                if !existed {
                    let data = self.animations.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            },
            "icon" => {
                let existed = self.icons.has(path_string.clone());
                let handle = self.icons.add(path_string);
                if !existed {
                    let data = self.icons.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            },
            "skin" => {
                let existed = self.skins.has(path_string.clone());
                let handle = self.skins.add(path_string);
                if !existed {
                    let data = self.skins.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            },
            "model" => {
                let existed = self.models.has(path_string.clone());
                let handle = self.models.add(path_string);
                if !existed {
                    let data = self.models.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            },
            "scene" => {
                let existed = self.scenes.has(path_string.clone());
                let handle = self.scenes.add(path_string);
                if !existed {
                    let data = self.scenes.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                let asset_handle: AssetHandle = handle.into();
                asset_handle.into()
            },
            _ => panic!("Unknown file extension: {}", file_ext),
        };

        if !dependencies.is_empty() {
            for (principal_handle, dependency_string) in dependencies {
                let dependency_handle = self.load(&dependency_string);
                self.finish_dependency(principal_handle, dependency_string, dependency_handle);
            }
        }

        asset_handle
    }

    fn finish_dependency(&mut self, principal_handle: AssetHandle, dependency_string: String, dependency_handle: AssetHandle) {
        match principal_handle.to_impl() {
            AssetHandleImpl::Mesh(_) | AssetHandleImpl::Skeleton(_) | AssetHandleImpl::Palette(_) => {
                panic!("unexpected dependency for this type of asset")
            },
            AssetHandleImpl::Animation(principal_handle) => {
                let mut data = self.animations.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);
            },
            AssetHandleImpl::Icon(principal_handle) => {
                let mut data = self.icons.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);
            },
            AssetHandleImpl::Skin(principal_handle) => {
                let mut data = self.skins.get_mut(&principal_handle).unwrap();
                if let Some(palette_handle) = data.finish_dependency(dependency_string, dependency_handle) {
                    if self.palette_has_cpu_materials(&palette_handle) {
                        self.ready_skins.push((principal_handle, palette_handle));
                    } else {
                        self.skins_waiting_on_palettes.insert(palette_handle, principal_handle);
                    }
                }
            },
            AssetHandleImpl::Model(principal_handle) => {
                let mut data = self.models.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);
            },
            AssetHandleImpl::Scene(principal_handle) => {
                let mut data = self.scenes.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);
            },
        }
    }

    pub fn sync(
        mut asset_manager: ResMut<Self>,
        mut meshes: ResMut<Assets<CpuMesh>>,
        mut materials: ResMut<Assets<CpuMaterial>>,
        mut skins: ResMut<Assets<CpuSkin>>,
    ) {
        asset_manager.sync_meshes(&mut meshes);
        asset_manager.sync_palettes(&mut materials);
        asset_manager.sync_skins(&materials, &mut skins);
    }

    fn sync_meshes(&mut self, meshes: &mut Assets<CpuMesh>) {
        if self.queued_meshes.is_empty() {
            return;
        }

        for mesh_handle in self.queued_meshes.drain(..) {
            let mesh_file = self.meshes.get_mut(&mesh_handle).unwrap();
            mesh_file.load_cpu_mesh(meshes);
        }
    }

    fn sync_palettes(&mut self, materials: &mut Assets<CpuMaterial>) {
        if self.queued_palettes.is_empty() {
            return;
        }

        let palette_handles = std::mem::take(&mut self.queued_palettes);

        for palette_handle in &palette_handles {
            let palette_data = self.palettes.get_mut(palette_handle).unwrap();
            palette_data.load_cpu_materials(materials);
        }

        for palette_handle in &palette_handles {
            if let Some(skin_handle) = self.skins_waiting_on_palettes.remove(palette_handle) {
                self.ready_skins.push((skin_handle, palette_handle.clone()));
            }
        }
    }

    fn sync_skins(&mut self, materials: &Assets<CpuMaterial>, skins: &mut Assets<CpuSkin>) {
        if self.ready_skins.is_empty() {
            return;
        }

        for (skin_handle, palette_handle) in std::mem::take(&mut self.ready_skins) {
            let skin_data = self.skins.get_mut(&skin_handle).unwrap();
            let palette_data = self.palettes.get(&palette_handle).unwrap();
            if skin_data.load_cpu_skin(materials, skins, palette_data) {
                // success!
            } else {
                warn!("skin data not loaded, re-queuing");
                self.ready_skins.push((skin_handle, palette_handle));
            }
        }
    }

    // Drawing

    pub fn draw_mesh(
        &self,
        render_frame: &mut RenderFrame,
        mesh_handle: &Handle<MeshFile>,
        mat_handle: &Handle<CpuMaterial>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        let Some(mesh_file) = self.meshes.get(mesh_handle) else {
            warn!("mesh file not loaded 1: {:?}", mesh_handle);
            return;
        };
        let Some(cpu_mesh_handle) = mesh_file.get_cpu_mesh_handle() else {
            warn!("mesh file not loaded 2: {:?}", mesh_handle);
            return;
        };
        render_frame.draw_mesh(render_layer_opt, cpu_mesh_handle, mat_handle, transform);
    }

    pub fn draw_skin(
        &self,
        render_frame: &mut RenderFrame,
        skin_handle: &Handle<SkinData>,
        transform: &Transform,
        render_layer_opt: Option<&RenderLayer>,
    ) {
        let Some(skin_data) = self.skins.get(skin_handle) else {
            warn!("skin data not loaded 1: {:?}", skin_handle);
            return;
        };
        let Some(mesh_file_handle) = skin_data.get_mesh_file_handle() else {
            warn!("skin file not loaded 2: {:?}", skin_handle);
            return;
        };
        let Some(mesh_file) = self.meshes.get(mesh_file_handle) else {
            warn!("skin file not loaded 3: {:?}", skin_handle);
            return;
        };
        let Some(cpu_mesh_handle) = mesh_file.get_cpu_mesh_handle() else {
            warn!("skin file not loaded 4: {:?}", skin_handle);
            return;
        };
        let Some(cpu_skin_handle) = skin_data.get_cpu_skin_handle() else {
            warn!("skin data not loaded 5: {:?}", skin_handle);
            return;
        };
        render_frame.draw_skinned_mesh(render_layer_opt, cpu_mesh_handle, cpu_skin_handle, transform);
    }

    fn palette_has_cpu_materials(&self, palette_handle: &Handle<PaletteData>) -> bool {
        let palette_data = self.palettes.get(palette_handle).unwrap();
        palette_data.has_cpu_materials()
    }
}