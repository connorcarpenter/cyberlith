use bevy_app::{App, Plugin};

use render_api::Assets;

use crate::{PaletteData, SkeletonData};

// Plugin
pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Assets<SkeletonData>>()
            .init_resource::<Assets<PaletteData>>();
    }
}
