use bevy_ecs::system::Resource;

use render_api::{Assets, Handle, base::{CpuMaterial, CpuMesh}};

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
        }
    }
}

impl AssetManager {
    pub fn load(&mut self, path: &str) -> AssetHandle {
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
                handle.into()
            },
            "skel" => self.skeletons.add(path_string).into(),
            "palette" => {
                let existed = self.palettes.has(path_string.clone());
                let handle = self.palettes.add(path_string);
                if !existed {
                    self.queued_palettes.push(handle.clone());
                }
                handle.into()
            },
            "anim" => {
                let existed = self.animations.has(path_string.clone());
                let handle = self.animations.add(path_string);
                if !existed {
                    let data = self.animations.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                handle.into()
            },
            "icon" => {
                let existed = self.icons.has(path_string.clone());
                let handle = self.icons.add(path_string);
                if !existed {
                    let data = self.icons.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                handle.into()
            },
            "skin" => {
                let existed = self.skins.has(path_string.clone());
                let handle = self.skins.add(path_string);
                if !existed {
                    let data = self.skins.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                handle.into()
            },
            "model" => {
                let existed = self.models.has(path_string.clone());
                let handle = self.models.add(path_string);
                if !existed {
                    let data = self.models.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                handle.into()
            },
            "scene" => {
                let existed = self.scenes.has(path_string.clone());
                let handle = self.scenes.add(path_string);
                if !existed {
                    let data = self.scenes.get(&handle).unwrap();
                    data.load_dependencies(handle, &mut dependencies);
                }
                handle.into()
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
            AssetHandleImpl::Mesh(principal_handle) => {
                let mut data = self.meshes.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);
            },
            AssetHandleImpl::Skeleton(principal_handle) => {
                let mut data = self.skeletons.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);
            },
            AssetHandleImpl::Palette(principal_handle) => {
                let mut data = self.palettes.get_mut(&principal_handle).unwrap();
                data.finish_dependency(dependency_string, dependency_handle);
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
                data.finish_dependency(dependency_string, dependency_handle);
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
}