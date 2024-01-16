use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_client::events::ErrorEvent;

use crate::app::plugin::Main;

pub fn error_events(mut event_reader: EventReader<ErrorEvent<Main>>) {
    for event in event_reader.read() {
        info!("Client Error: {:?}", event.err);
    }
}
