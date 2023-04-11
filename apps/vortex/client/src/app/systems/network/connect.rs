use bevy_ecs::{event::EventReader, system::ResMut};
use bevy_log::info;

use naia_bevy_client::{events::ConnectEvent, Client};

use crate::app::ui::UiState;

pub fn connect_events(client: Client, mut event_reader: EventReader<ConnectEvent>, mut state: ResMut<UiState>) {
    for _ in event_reader.iter() {
        let server_address = client.server_address().unwrap();
        info!("Client connected to: {}", server_address);
        state.logged_in = true;
    }
}
