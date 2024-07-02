use bevy_ecs::{change_detection::ResMut, event::EventReader, system::Res};

use naia_bevy_server::events::MessageEvents;

use bevy_http_client::HttpClient;

use logging::warn;

use session_server_naia_proto::{
    channels::ClientActionsChannel,
    messages::{GlobalChatSendMessage, WorldConnectRequest},
};

use crate::{session_instance::SessionInstance, social::SocialManager, user::UserManager};

pub fn message_events(
    mut http_client: ResMut<HttpClient>,
    mut user_manager: ResMut<UserManager>,
    mut social_manager: ResMut<SocialManager>,
    session_instance: Res<SessionInstance>,
    mut event_reader: EventReader<MessageEvents>,
) {
    for events in event_reader.read() {
        // World Connect Request
        for (user_key, _req) in events.read::<ClientActionsChannel, WorldConnectRequest>() {
            if user_manager.make_ready_for_world_connect(&user_key).is_err() {
                warn!("User not found: {:?}", user_key);
            }
        }

        // Global Chat Send Message
        for (user_key, req) in events.read::<ClientActionsChannel, GlobalChatSendMessage>() {
            social_manager.send_global_chat_message(
                &mut http_client,
                &user_manager,
                &session_instance,
                &user_key,
                &req.message,
            );
        }
    }
}
