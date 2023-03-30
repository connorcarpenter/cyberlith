use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{apply_system_buffers, IntoSystemConfig, IntoSystemSetConfig};

use crate::{
    assets::{Assets, Image, Material, Mesh},
    base_set::RenderSet,
    Window,
};

pub struct RenderApiPlugin;

impl Plugin for RenderApiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            // TODO: find out how to get window height & width
            .insert_resource(Window::new(1280, 720))
            .insert_resource(Assets::<Mesh>::default())
            .insert_resource(Assets::<Material>::default())
            .insert_resource(Assets::<Image>::default())
            // Sync
            .configure_set(RenderSet::Sync.after(CoreSet::LastFlush))
            // SyncFlush
            .configure_set(RenderSet::SyncFlush.after(RenderSet::Sync))
            .add_system(apply_system_buffers.in_base_set(RenderSet::SyncFlush))
            // Draw
            .configure_set(RenderSet::Draw.after(RenderSet::SyncFlush));
    }
}
