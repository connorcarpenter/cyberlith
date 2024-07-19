use bevy_ecs::{change_detection::ResMut, event::EventReader, system::Res};

use naia_bevy_server::events::MessageEvents;

use bevy_http_client::HttpClient;

use session_server_naia_proto::{
    channels::ClientActionsChannel,
    messages::{GlobalChatSendMessage, MatchLobbyGameStart, MatchLobbyLeave, MatchLobbyJoin, MatchLobbySendMessage, MatchLobbyCreate},
};

use crate::{session_instance::SessionInstance, social::SocialManager, user::UserManager};

pub fn message_events(
    mut http_client: ResMut<HttpClient>,
    user_manager: Res<UserManager>,
    mut social_manager: ResMut<SocialManager>,
    session_instance: Res<SessionInstance>,
    mut event_reader: EventReader<MessageEvents>,
) {
    for events in event_reader.read() {

        let social_server_url = social_manager.get_social_server_url();

        // Global Chat Send Message
        for (user_key, req) in events.read::<ClientActionsChannel, GlobalChatSendMessage>() {
            social_manager
                .chat_message_manager
                .send_global_chat_message(
                    &mut http_client,
                    &user_manager,
                    social_server_url.as_ref(),
                    &session_instance,
                    &user_key,
                    &req.message,
                );
        }

        // Lobby Send Message
        for (user_key, req) in events.read::<ClientActionsChannel, MatchLobbySendMessage>() {
            social_manager
                .chat_message_manager
                .send_lobby_chat_message(

                    &mut http_client,
                    &user_manager,
                    social_server_url.as_ref(),
                    &session_instance,
                    &user_key,
                    &req.message,
                );
        }

        // Create Match Lobby
        for (user_key, req) in events.read::<ClientActionsChannel, MatchLobbyCreate>() {
            social_manager.lobby_manager.send_match_lobby_create(
                &mut http_client,
                &user_manager,
                social_server_url.as_ref(),
                &session_instance,
                &user_key,
                &req.match_name,
            );
        }

        // Join Match Lobby
        for (user_key, req) in events.read::<ClientActionsChannel, MatchLobbyJoin>() {
            social_manager.lobby_manager.send_match_lobby_join(
                &mut http_client,
                &user_manager,
                social_server_url.as_ref(),
                &session_instance,
                &user_key,
                &req.match_id,
            );
        }

        // Start Match
        for (user_key, _req) in events.read::<ClientActionsChannel, MatchLobbyGameStart>() {
            social_manager.lobby_manager.send_match_lobby_start(
                &mut http_client,
                &user_manager,
                social_server_url.as_ref(),
                &session_instance,
                &user_key,
            );
        }

        // Leave Match Lobby
        for (user_key, _req) in events.read::<ClientActionsChannel, MatchLobbyLeave>() {
            social_manager.lobby_manager.send_match_lobby_leave(
                &mut http_client,
                &user_manager,
                social_server_url.as_ref(),
                &session_instance,
                &user_key,
            );
        }
    }
}
