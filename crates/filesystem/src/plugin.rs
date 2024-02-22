use bevy_app::{App, Plugin, Update};

use crate::manager::{update, FileSystemManager};

#[derive(Default)]
pub struct FileSystemPlugin;

impl Plugin for FileSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_core::TaskPoolPlugin::default())
            .init_resource::<FileSystemManager>()
            .add_systems(Update, update);
    }
}
