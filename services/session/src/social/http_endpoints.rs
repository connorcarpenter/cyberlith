use bevy_ecs::{change_detection::ResMut, system::Commands};

use naia_bevy_server::Server;

use bevy_http_client::ResponseError;
use bevy_http_server::HttpServer;
use config::SOCIAL_SERVER_GLOBAL_SECRET;
use logging::warn;

use session_server_http_proto::{
    SocialPatchGlobalChatMessagesRequest, SocialPatchGlobalChatMessagesResponse,
};

use crate::social::SocialManager;

pub fn recv_patch_global_chat_messages_request(
    mut commands: Commands,
    mut social_manager: ResMut<SocialManager>,
    mut http_server: ResMut<HttpServer>,
    mut naia_server: Server,
) {
    while let Some((_addr, request, response_key)) =
        http_server.receive::<SocialPatchGlobalChatMessagesRequest>()
    {
        if request.social_secret() != SOCIAL_SERVER_GLOBAL_SECRET {
            warn!("invalid request secret");
            http_server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        social_manager.patch_global_chat_messages(
            &mut commands,
            &mut naia_server,
            request.new_messages(),
        );

        // responding
        http_server.respond(response_key, Ok(SocialPatchGlobalChatMessagesResponse));
    }
}
