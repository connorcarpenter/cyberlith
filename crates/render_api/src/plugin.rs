use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{apply_system_buffers, IntoSystemConfig, IntoSystemSetConfig};

use crate::{
    assets::Assets,
    base::{CpuMaterial, CpuMesh, CpuTexture2D},
    base_set::RenderSet,
    Window,
};

pub struct RenderApiPlugin;

impl Plugin for RenderApiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(Assets::<CpuMesh>::default())
            .insert_resource(Assets::<CpuMaterial>::default())
            .insert_resource(Assets::<CpuTexture2D>::default())
            // TODO: figure out how to set the correct window here ...
            .insert_resource(Window::default())
            // Sync
            .configure_set(RenderSet::Sync.after(CoreSet::LastFlush))
            // SyncFlush
            .configure_set(RenderSet::SyncFlush.after(RenderSet::Sync))
            .add_system(apply_system_buffers.in_base_set(RenderSet::SyncFlush))
            // Draw
            .configure_set(RenderSet::Draw.after(RenderSet::SyncFlush));
    }
}
