use bevy_app::{App, Plugin};

use crate::{
    assets::{Assets, Image, Mesh, StandardMaterial},
    runner::three_d_runner,
    Window,
};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            // TODO: find out how to get window height & width
            .insert_resource(Window::new(0.0, 0.0))
            .insert_resource(Assets::<Mesh>::default())
            .insert_resource(Assets::<StandardMaterial>::default())
            .insert_resource(Assets::<Image>::default())
            // Runner for Three-D integration
            .set_runner(three_d_runner);
    }
}
