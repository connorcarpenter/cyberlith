use bevy_ecs::{event::EventReader, system::ResMut};
use bevy_log::info;

use naia_bevy_client::{events::ConnectEvent, Client};

use crate::app::{
    plugin::Main,
    ui::{LoggingInState, UiState},
};

pub fn connect_events(
    client: Client<Main>,
    mut event_reader: EventReader<ConnectEvent<Main>>,
    mut state: ResMut<UiState>,
) {
    for _ in event_reader.read() {
        let server_address = client.server_address().unwrap();
        info!("Client connected to: {}", server_address);
        state.logged_in = true;
        state.logging_in_state = LoggingInState::NotLoggingIn;
    }
}
