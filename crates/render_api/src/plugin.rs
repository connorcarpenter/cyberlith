use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{apply_system_buffers, IntoSystemConfig, IntoSystemSetConfig};

use crate::{
    assets::Assets,
    base::{PbrMaterial, Texture2D, TriMesh},
    base_set::RenderSet,
    Window,
};

pub struct RenderApiPlugin;

impl Plugin for RenderApiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(Assets::<TriMesh>::default())
            .insert_resource(Assets::<PbrMaterial>::default())
            .insert_resource(Assets::<Texture2D>::default())
            // Sync
            .configure_set(RenderSet::Sync.after(CoreSet::LastFlush))
            // SyncFlush
            .configure_set(RenderSet::SyncFlush.after(RenderSet::Sync))
            .add_system(apply_system_buffers.in_base_set(RenderSet::SyncFlush))
            // Draw
            .configure_set(RenderSet::Draw.after(RenderSet::SyncFlush));
    }
}
