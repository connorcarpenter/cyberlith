use bevy_ecs::system::Resource;

use render_api::{Assets, Handle, base::{CpuMaterial, CpuMesh}};

use crate::{AnimationData, AssetHandle, IconData, ModelData, PaletteData, SceneData, SkeletonData, SkinData};

#[derive(Resource)]
pub struct AssetManager {
    skeletons: Assets<SkeletonData>,
    palettes: Assets<PaletteData>,
    animations: Assets<AnimationData>,
    icons: Assets<IconData>,
    skins: Assets<SkinData>,
    models: Assets<ModelData>,
    scenes: Assets<SceneData>,

    // mesh file name, skin handle
    queued_meshes: Vec<(String, Handle<SkinData>)>,
    queued_materials: Vec<Handle<PaletteData>>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            skeletons: Assets::default(),
            palettes: Assets::default(),
            animations: Assets::default(),
            icons: Assets::default(),
            skins: Assets::default(),
            models: Assets::default(),
            scenes: Assets::default(),

            queued_meshes: Vec::new(),
            queued_materials: Vec::new(),
        }
    }
}

impl AssetManager {
    pub fn load(&mut self, path: &str) -> AssetHandle {
        let file_ext = path.split('.').last().unwrap();
        let path_string = path.to_string();

        let mut dependencies = Vec::new();

        let asset_handle = match file_ext {
            "skel" => self.skeletons.add(path_string).into(),
            "palette" => {
                let existed = self.palettes.has(path_string.clone());
                let handle = self.palettes.add(path_string);
                if !existed {
                    self.queued_materials.push(handle.clone());
                }
                handle.into()
            },
            "anim" => {
                let existed = self.animations.has(path_string.clone());
                let handle = self.animations.add(path_string);
                if !existed {
                    let data = self.animations.get(&handle).unwrap();
                    data.load_dependencies(&mut dependencies);
                }
                handle.into()
            },
            "icon" => {
                let existed = self.icons.has(path_string.clone());
                let handle = self.icons.add(path_string);
                if !existed {
                    let data = self.icons.get(&handle).unwrap();
                    data.load_dependencies(&mut dependencies);
                }
                handle.into()
            },
            "skin" => {
                let existed = self.skins.has(path_string.clone());
                let handle = self.skins.add(path_string);
                if !existed {
                    let data = self.skins.get(&handle).unwrap();
                    data.load_dependencies(&mut dependencies);
                    self.queued_meshes.push((data.mesh_file_path().to_string(), handle.clone()));
                }
                handle.into()
            },
            "model" => {
                let existed = self.models.has(path_string.clone());
                let handle = self.models.add(path_string);
                if !existed {
                    let data = self.models.get(&handle).unwrap();
                    data.load_dependencies(&mut dependencies);
                }
                handle.into()
            },
            "scene" => {
                let existed = self.scenes.has(path_string.clone());
                let handle = self.scenes.add(path_string);
                if !existed {
                    let data = self.scenes.get(&handle).unwrap();
                    data.load_dependencies(&mut dependencies);
                }
                handle.into()
            },
            _ => panic!("Unknown file extension: {}", file_ext),
        };

        if !dependencies.is_empty() {
            for dependency in dependencies {
                self.load(&dependency);
            }
        }

        asset_handle
    }
}