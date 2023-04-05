use bevy_app::{App, Plugin};
use bevy_ecs::schedule::{IntoSystemConfig, IntoSystemSetConfig};

use render_api::{RenderApiPlugin, RenderSet};
use render_glow::RenderGlowPlugin;

use crate::{draw, EguiDrawSet};

// Plugin
pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenderApiPlugin)
            .add_plugin(RenderGlowPlugin)
            .configure_set(EguiDrawSet.after(RenderSet::Draw))
            .add_system(draw.in_base_set(EguiDrawSet));
    }
}
