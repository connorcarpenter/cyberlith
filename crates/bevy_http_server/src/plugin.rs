use std::ops::DerefMut;
use std::sync::Mutex;

use bevy_app::{App, Plugin, Update};

use bevy_http_shared::Protocol;

use crate::server::HttpServer;

pub struct HttpServerPlugin {
    protocol: Mutex<Option<Protocol>>
}

impl HttpServerPlugin {
    pub fn new(protocol: Protocol) -> Self {
        Self {
            protocol: Mutex::new(Some(protocol)),
        }
    }
}

impl Plugin for HttpServerPlugin {
    fn build(&self, app: &mut App) {
        let protocol = self.protocol.lock().unwrap().deref_mut().take().unwrap();

        app.add_plugins(bevy_core::TaskPoolPlugin::default())
            .insert_resource(HttpServer::new(protocol));
    }
}