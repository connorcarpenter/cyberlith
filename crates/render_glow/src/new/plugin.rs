use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{IntoSystemSetConfig, IntoSystemConfig};

use render_api::{
    Assets, Image, Mesh, StandardMaterial,
    Window,
};

use crate::new::{
    runner::three_d_runner,
    systems::{RenderSet, draw},
};

pub struct RenderGlowPlugin;

impl Plugin for RenderGlowPlugin {
    fn build(&self, app: &mut App) {
        app
            // Runner for Three-D integration
            .set_runner(three_d_runner)
            // Base System Set
            .configure_set(RenderSet::Draw.after(CoreSet::LastFlush))
            // Systems
            .add_system(draw.in_base_set(RenderSet::Draw));
    }
}
