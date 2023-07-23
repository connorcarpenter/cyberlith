use bevy_ecs::{event::EventReader, system::ResMut};
use bevy_log::info;

use crate::resources::UserManager;
use naia_bevy_server::{
    events::{PublishEntityEvent, UnpublishEntityEvent},
    Server,
};

pub fn publish_entity_events(
    mut server: Server,
    user_manager: ResMut<UserManager>,
    mut event_reader: EventReader<PublishEntityEvent>,
) {
    for PublishEntityEvent(user_key, client_entity) in event_reader.iter() {
        info!("client entity has been made public: {:?}", client_entity);

        let room_key = user_manager
            .user_info(user_key)
            .unwrap()
            .get_room_key()
            .unwrap();

        // Add newly public entity to the main Room
        server.room_mut(&room_key).add_entity(client_entity);
    }
}

pub fn unpublish_entity_events(mut event_reader: EventReader<UnpublishEntityEvent>) {
    for UnpublishEntityEvent(_user_key, client_entity) in event_reader.iter() {
        info!("client entity has been unpublished: {:?}", client_entity);
    }
}
