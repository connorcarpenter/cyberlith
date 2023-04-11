use bevy_ecs::{event::EventReader, system::ResMut};
use bevy_log::info;

use naia_bevy_client::events::RejectEvent;

use crate::app::ui::{LoggingInState, UiState};

pub fn reject_events(mut event_reader: EventReader<RejectEvent>, mut state: ResMut<UiState>) {
    for _ in event_reader.iter() {
        info!("Client rejected from connecting to Server");
        state.logging_in_state = LoggingInState::LoginFailed;
    }
}
