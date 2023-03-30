use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{IntoSystemConfig, IntoSystemSetConfig};

use render_api::{Assets, Image, Mesh, Material, Window, RenderSet};

use crate::bevy::{
    runner::three_d_runner,
    systems::draw,
};

pub struct RenderGlowPlugin;

impl Plugin for RenderGlowPlugin {
    fn build(&self, app: &mut App) {
        app
            // Runner for Three-D integration
            .set_runner(three_d_runner)
            // Systems
            .add_system(draw.in_base_set(RenderSet::Draw));
    }
}
