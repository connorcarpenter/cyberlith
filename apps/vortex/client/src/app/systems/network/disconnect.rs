use bevy_ecs::{event::EventReader, system::ResMut};
use bevy_log::info;

use naia_bevy_client::events::DisconnectEvent;

use crate::app::ui::{LoggingInState, UiState};

pub fn disconnect_events(
    mut event_reader: EventReader<DisconnectEvent>,
    mut state: ResMut<UiState>,
) {
    for _ in event_reader.read() {
        info!("Client disconnected from Server");
        state.logged_in = false;
        state.logging_in_state = LoggingInState::NotLoggingIn;
    }
}
