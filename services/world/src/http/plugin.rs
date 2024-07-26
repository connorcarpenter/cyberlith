use bevy_app::{App, Plugin, Startup, Update};

use bevy_http_client::HttpClientPlugin;
use bevy_http_server::HttpServerPlugin;

use world_server_http_proto::protocol as http_protocol;

use crate::http::systems;

pub struct HttpPlugin;

impl Plugin for HttpPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugins(HttpServerPlugin::new(http_protocol()))
            .add_plugins(HttpClientPlugin)
            // Systems
            .add_systems(Startup, systems::init)
            .add_systems(Update, systems::recv_world_connect_request);
    }
}
