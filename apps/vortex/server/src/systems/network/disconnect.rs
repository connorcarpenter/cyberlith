use bevy_ecs::{event::EventReader, system::ResMut};
use bevy_log::info;

use naia_bevy_server::events::DisconnectEvent;

use crate::resources::UserManager;

pub fn disconnect_events(
    mut event_reader: EventReader<DisconnectEvent>,
    mut user_manager: ResMut<UserManager>,
) {
    for DisconnectEvent(user_key, user) in event_reader.read() {
        info!("Server disconnected from: {:?}", user.address);

        user_manager.logout_user(user_key);
    }
}
