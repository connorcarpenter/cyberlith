use bevy_ecs::system::Resource;

use render_api::Assets;

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
        }
    }
}

impl AssetManager {
    pub fn load(&mut self, path: &str) -> AssetHandle {
        let file_ext = path.split('.').last().unwrap();
        let path_string = path.to_string();
        match file_ext {
            "skel" => self.skeletons.add(path_string).into(),
            "palette" => self.palettes.add(path_string).into(),
            "anim" => self.animations.add(path_string).into(),
            "icon" => self.icons.add(path_string).into(),
            "skin" => self.skins.add(path_string).into(),
            "model" => self.models.add(path_string).into(),
            "scene" => self.scenes.add(path_string).into(),
            _ => panic!("Unknown file extension: {}", file_ext),
        }
    }
}