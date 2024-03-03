use bevy_app::{App, Plugin, Update};

use crate::manager::{update, FileSystemManager};

#[derive(Default)]
pub struct FileSystemPlugin;

impl Plugin for FileSystemPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_core::TaskPoolPlugin>() {
            app.add_plugins(bevy_core::TaskPoolPlugin::default());
        }
        app.init_resource::<FileSystemManager>()
            .add_systems(Update, update);
    }
}
