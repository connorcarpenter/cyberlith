use bevy_app::{App, Plugin, Update};

use crate::client::HttpClient;

#[derive(Default)]
pub struct HttpClientPlugin;

impl HttpClientPlugin {
    pub fn new() -> Self {
        panic!("HttpClientPlugin::new() is not supported in wasm!");
    }
}

impl Plugin for HttpClientPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_core::TaskPoolPlugin>() {
            app.add_plugins(bevy_core::TaskPoolPlugin::default());
        }
        app.init_resource::<HttpClient>()
            .add_systems(Update, HttpClient::update_system);
    }
}
