use bevy_app::{App, Plugin, Startup};

use bevy_http_client::HttpClientPlugin;
use bevy_http_server::HttpServerPlugin;

use session_server_http_proto::protocol as http_protocol;

use crate::http::server_startup;

pub struct HttpPlugin {}

impl HttpPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for HttpPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HttpServerPlugin::new(http_protocol()))
            .add_plugins(HttpClientPlugin)
            .add_systems(Startup, server_startup::system);
    }
}
