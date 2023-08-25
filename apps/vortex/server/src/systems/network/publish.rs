use bevy_ecs::{event::EventReader, system::ResMut};
use bevy_log::info;

use crate::resources::UserManager;
use naia_bevy_server::{
    events::{PublishEntityEvent, UnpublishEntityEvent},
    Server,
};

pub fn publish_entity_events(
    mut event_reader: EventReader<PublishEntityEvent>,
) {
    for PublishEntityEvent(_user_key, client_entity) in event_reader.iter() {
        info!("client entity has been made public: {:?}", client_entity);
    }
}

pub fn unpublish_entity_events(mut event_reader: EventReader<UnpublishEntityEvent>) {
    for UnpublishEntityEvent(_user_key, client_entity) in event_reader.iter() {
        info!("client entity has been unpublished: {:?}", client_entity);

        // TODO: remove entity from main Room
    }
}
