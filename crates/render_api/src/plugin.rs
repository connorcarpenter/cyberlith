use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{IntoSystemSetConfig, IntoSystemConfig};

use crate::{
    assets::{Assets, Image, Mesh, StandardMaterial},
    runner::three_d_runner,
    Window,
    systems::{RenderSet, draw},
};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            // TODO: find out how to get window height & width
            .insert_resource(Window::new(1280, 720))
            .insert_resource(Assets::<Mesh>::default())
            .insert_resource(Assets::<StandardMaterial>::default())
            .insert_resource(Assets::<Image>::default())
            // Runner for Three-D integration
            .set_runner(three_d_runner)
            // Base System Set
            .configure_set(RenderSet::Draw.after(CoreSet::LastFlush))
            // Systems
            .add_system(draw.in_base_set(RenderSet::Draw))
            ;
    }
}
