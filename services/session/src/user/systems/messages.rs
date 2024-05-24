
use bevy_ecs::{system::{Res, Commands}, change_detection::ResMut, event::EventReader};

use naia_bevy_server::{events::{MessageEvents, TickEvent}, Server};

use bevy_http_client::HttpClient;

use logging::warn;

use session_server_naia_proto::{channels::ClientActionsChannel, messages::{GlobalChatSendMessage, WorldConnectRequest}};

use crate::{session_instance::SessionInstance, user::UserManager, social::SocialManager};

pub fn message_events(
    mut commands: Commands,
    mut naia_server: Server,
    mut http_client: ResMut<HttpClient>,
    mut user_manager: ResMut<UserManager>,
    mut social_manager: ResMut<SocialManager>,
    session_instance: Res<SessionInstance>,
    mut event_reader: EventReader<MessageEvents>
) {
    for events in event_reader.read() {

        // World Connect Request
        for (user_key, _req) in events.read::<ClientActionsChannel, WorldConnectRequest>() {
            if let Some(user_data) = user_manager.get_user_data_mut(&user_key) {
                user_data.make_ready_for_world_connect();
            } else {
                warn!("User not found: {:?}", user_key);
            }
        }

        // Global Chat Send Message
        for (user_key, req) in events.read::<ClientActionsChannel, GlobalChatSendMessage>() {
            social_manager.send_global_chat_message(
                &mut commands,
                &mut naia_server,
                &mut http_client,
                &user_manager,
                &session_instance,
                &user_key,
                &req.message
            );
        }
    }
}