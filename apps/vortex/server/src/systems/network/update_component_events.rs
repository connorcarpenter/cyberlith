
use bevy_ecs::{
    event::EventReader,
    system::{Commands, ResMut},
};

use naia_bevy_server::{
    events::UpdateComponentEvents,
    Server,
};

use vortex_proto::components::{
        FileSystemChild, FileSystemEntry,
        Vertex3d,
    };

use crate::resources::GitManager;

pub fn update_component_events(
    mut event_reader: EventReader<UpdateComponentEvents>,
    mut commands: Commands,
    mut server: Server,
    mut git_manager: ResMut<GitManager>,
) {
    for events in event_reader.iter() {
        // on FileSystemEntry Update Event
        for (_user_key, _entity) in events.read::<FileSystemEntry>() {
            // TODO!
        }
        // on FileSystemChild Update Event
        for (_user_key, _entity) in events.read::<FileSystemChild>() {
            // TODO!
        }
        // on Vertex3D Update Event
        for (_, entity) in events.read::<Vertex3d>() {
            let Some((project_key, file_key)) = git_manager.content_entity_keys(&entity) else {
                panic!("no content entity keys!");
            };
            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }
    }
}
