use bevy_ecs::event::EventReader;

use naia_bevy_server::events::{PublishEntityEvent, UnpublishEntityEvent};

pub fn publish_entity_events(mut event_reader: EventReader<PublishEntityEvent>) {
    for PublishEntityEvent(_user_key, _client_entity) in event_reader.iter() {
        //info!("client entity has been made public: {:?}", client_entity);
    }
}

pub fn unpublish_entity_events(mut event_reader: EventReader<UnpublishEntityEvent>) {
    for UnpublishEntityEvent(_user_key, _client_entity) in event_reader.iter() {
        //info!("client entity has been unpublished: {:?}", client_entity);

        // TODO: remove entity from main Room
    }
}
