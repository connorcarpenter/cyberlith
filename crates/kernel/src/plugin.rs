use bevy_app::{App, Plugin};

use crate::{AppExitAction, http::HttpClientPlugin};

pub struct KernelPlugin;

impl Plugin for KernelPlugin {
    fn build(&self, app: &mut App) {

        app.add_event::<AppExitAction>().add_plugins(HttpClientPlugin);
    }
}
