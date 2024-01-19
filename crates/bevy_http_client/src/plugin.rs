
use bevy_app::{Plugin, App, Update};

use crate::client::{client_update, HttpClient};

#[derive(Default)]
pub struct HttpClientPlugin;

impl Plugin for HttpClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(bevy_core::TaskPoolPlugin::default())
            .init_resource::<HttpClient>()
            .add_systems(Update, client_update);
    }
}