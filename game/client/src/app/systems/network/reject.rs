use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_client::events::RejectEvent;

pub fn reject_events(mut event_reader: EventReader<RejectEvent>) {
    for _ in event_reader.iter() {
        info!("Client rejected from connecting to Server");
    }
}
