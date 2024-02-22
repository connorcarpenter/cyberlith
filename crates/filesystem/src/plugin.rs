use bevy_app::{App, Plugin, Update};

use crate::client::{client_update, FileSystemClient};

#[derive(Default)]
pub struct FileSystemPlugin;

impl Plugin for FileSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_core::TaskPoolPlugin::default())
            .init_resource::<FileSystemClient>()
            .add_systems(Update, client_update);
    }
}
