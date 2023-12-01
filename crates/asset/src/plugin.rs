use bevy_app::{App, Plugin};

use render_api::Assets;

use crate::{PaletteData, SkeletonData, AnimationData, IconData, SkinData, ModelData, SceneData};

// Plugin
pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Assets<SkeletonData>>()
            .init_resource::<Assets<PaletteData>>()
            .init_resource::<Assets<AnimationData>>()
            .init_resource::<Assets<IconData>>()
            .init_resource::<Assets<SkinData>>()
            .init_resource::<Assets<ModelData>>()
            .init_resource::<Assets<SceneData>>();
    }
}
