use bevy_app::{App, Plugin};

use render_gl::RenderglPlugin;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenderglPlugin);
    }
}
