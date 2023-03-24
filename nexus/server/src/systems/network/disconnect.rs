use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_server::events::DisconnectEvent;

pub fn disconnect_events(mut event_reader: EventReader<DisconnectEvent>) {
    for DisconnectEvent(_user_key, user) in event_reader.iter() {
        info!("Server disconnected from: {:?}", user.address);
    }
}
