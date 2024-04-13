use bevy_ecs::event::EventReader;
use logging::info;

use naia_bevy_server::events::ErrorEvent;

pub fn error_events(mut event_reader: EventReader<ErrorEvent>) {
    for ErrorEvent(error) in event_reader.read() {
        info!("Server Error: {:?}", error);
    }
}
