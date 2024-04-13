use bevy_ecs::{event::EventReader, system::ResMut};
use logging::info;

use naia_bevy_client::events::RejectEvent;

use crate::app::{
    plugin::Main,
    ui::{LoggingInState, UiState},
};

pub fn reject_events(mut event_reader: EventReader<RejectEvent<Main>>, mut state: ResMut<UiState>) {
    for _ in event_reader.read() {
        info!("Client rejected from connecting to Server");
        state.logging_in_state = LoggingInState::LoginFailed;
    }
}
