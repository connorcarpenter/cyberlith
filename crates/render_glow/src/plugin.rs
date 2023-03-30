use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{apply_system_buffers, IntoSystemConfig, IntoSystemSetConfig};

use render_api::{Assets, RenderSet};

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
