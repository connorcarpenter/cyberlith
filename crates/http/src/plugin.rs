
use bevy_app::{Plugin, App, Update};

use crate::backend::{handle_request, handle_response};

#[derive(Default)]
pub struct HttpClientPlugin;

impl Plugin for HttpClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(bevy_core::TaskPoolPlugin::default())
            .add_systems(Update, (handle_request, handle_response));
    }
}