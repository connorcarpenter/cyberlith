use bevy_app::{App, Plugin};

use render_glow::RenderGlowPlugin;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenderGlowPlugin);
    }
}
