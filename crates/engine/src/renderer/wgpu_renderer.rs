use bevy_app::{App, Plugin};

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        // haven't implemented wgpu renderer yet!
        todo!();
    }
}