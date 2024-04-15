use bevy_app::{App, Plugin, Update};

use crate::client::{client_update, HttpClient};

#[derive(Default)]
pub struct HttpClientPlugin;

impl Plugin for HttpClientPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_core::TaskPoolPlugin>() {
            app.add_plugins(bevy_core::TaskPoolPlugin::default());
        }
        app
            .init_resource::<HttpClient>()
            .add_systems(Update, client_update);
    }
}
