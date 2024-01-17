
use bevy_app::{Plugin, App, Update};

use crate::{handle_request, handle_response};

#[derive(Default)]
pub struct HttpClientPlugin;

impl Plugin for crate::HttpClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_request, handle_response));
        app.add_plugins(bevy_core::TaskPoolPlugin::default());
    }
}