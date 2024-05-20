use bevy_ecs::change_detection::ResMut;

use bevy_http_client::ResponseError;
use bevy_http_server::HttpServer;
use config::SOCIAL_SERVER_GLOBAL_SECRET;
use logging::warn;

use session_server_http_proto::{SocialPatchGlobalChatMessagesRequest, SocialPatchGlobalChatMessagesResponse};

use crate::{social::SocialManager};

pub fn recv_patch_global_chat_messages_request(
    mut social_manager: ResMut<SocialManager>,
    mut server: ResMut<HttpServer>
) {
    while let Some((_addr, request, response_key)) = server.receive::<SocialPatchGlobalChatMessagesRequest>() {
        if request.social_secret() != SOCIAL_SERVER_GLOBAL_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        social_manager.patch_global_chat_messages(request.new_messages());

        // responding
        server.respond(response_key, Ok(SocialPatchGlobalChatMessagesResponse));
    }
}