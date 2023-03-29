use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{IntoSystemConfig, IntoSystemSetConfig};

use crate::{
    assets::{Assets, Image, Mesh, StandardMaterial},
    Window,
};

pub struct RenderApiPlugin;

impl Plugin for RenderApiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            // TODO: find out how to get window height & width
            .insert_resource(Window::new(1280, 720))
            .insert_resource(Assets::<Mesh>::default())
            .insert_resource(Assets::<StandardMaterial>::default())
            .insert_resource(Assets::<Image>::default());
    }
}
