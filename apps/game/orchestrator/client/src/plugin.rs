use bevy_app::{App, Plugin};

use http::HttpClientPlugin;

use crate::client::OrchestratorClientState;

pub struct OrchestratorClientPlugin;

impl Plugin for OrchestratorClientPlugin {
    fn build(&self, app: &mut App) {

        app.init_resource::<OrchestratorClientState>();

        if !app.is_plugin_added::<HttpClientPlugin>() {
            app.add_plugins(HttpClientPlugin);
        }
    }
}
