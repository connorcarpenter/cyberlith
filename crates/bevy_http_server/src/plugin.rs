
use bevy_app::{App, Plugin, Update};

use crate::server::{server_update, HttpServer};

#[derive(Default)]
pub struct HttpServerPlugin;

impl Plugin for HttpServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_core::TaskPoolPlugin::default())
            .init_resource::<HttpServer>()
            .add_systems(Update, server_update);
    }
}
