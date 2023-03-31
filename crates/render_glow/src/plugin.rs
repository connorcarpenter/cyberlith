use bevy_app::{App, Plugin};
use bevy_ecs::schedule::IntoSystemConfig;

use render_api::RenderSet;

use crate::{draw::draw, runner::three_d_runner, sync::SyncPlugin};

pub struct RenderGlowPlugin;

impl Plugin for RenderGlowPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugin(SyncPlugin)
            // Runner for Three-D integration
            .set_runner(three_d_runner)
            // Systems
            .add_system(draw.in_base_set(RenderSet::Draw));
    }
}
