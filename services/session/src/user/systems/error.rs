use bevy_ecs::event::EventReader;

use naia_bevy_server::events::ErrorEvent;

use logging::info;

pub fn error_events(mut event_reader: EventReader<ErrorEvent>) {
    for ErrorEvent(error) in event_reader.read() {
        info!("Server Error: {:?}", error);
    }
}
