use bevy_ecs::{
    event::EventReader,
    system::ResMut,
};
use bevy_log::info;

use naia_bevy_client::{events::MessageEvents, Client};

use vortex_proto::{
    channels::FileActionChannel,
    messages::FileBindMessage,
};

use crate::app::resources::file_manager::FileManager;

pub fn message_events(
    client: Client,
    mut event_reader: EventReader<MessageEvents>,
    mut file_manager: ResMut<FileManager>,
) {
    for events in event_reader.iter() {

        // File Bind Message
        for message in events.read::<FileActionChannel, FileBindMessage>() {
            let file_entity = message.file_entity.get(&client).unwrap();
            let dependency_entity = message.dependency_entity.get(&client).unwrap();

            file_manager.file_add_dependency(&file_entity, &dependency_entity);

            info!("received FileBindMessage(file: `{:?}`, dependency: `{:?}`)", file_entity, dependency_entity);
        }
    }
}
