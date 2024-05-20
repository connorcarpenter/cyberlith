
use bevy_ecs::{change_detection::ResMut, event::EventReader};

use naia_bevy_server::events::MessageEvents;

use logging::warn;

use session_server_naia_proto::channels::ClientActionsChannel;
use session_server_naia_proto::messages::WorldConnectRequest;

use crate::user::UserManager;

pub fn message_events(mut user_manager: ResMut<UserManager>, mut event_reader: EventReader<MessageEvents>) {
    for events in event_reader.read() {
        for (user_key, _req) in events.read::<ClientActionsChannel, WorldConnectRequest>() {
            if let Some(user_data) = user_manager.get_user_data_mut(&user_key) {
                user_data.make_ready_for_world_connect();
            } else {
                warn!("User not found: {:?}", user_key);
            }
        }
    }
}
