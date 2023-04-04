use bevy_app::Plugin;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // haven't implemented wgpu renderer yet!
        todo!();
    }
}
