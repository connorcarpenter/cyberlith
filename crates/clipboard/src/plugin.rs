
use bevy_app::{App, Plugin, Update};

use crate::{ClipboardManager, manager};

#[derive(Default)]
pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_core::TaskPoolPlugin>() {
            app.add_plugins(bevy_core::TaskPoolPlugin::default());
        }
        app
            .init_resource::<ClipboardManager>()
            .add_systems(Update, manager::update);
    }
}