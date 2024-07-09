use bevy_ecs::{prelude::Query, change_detection::ResMut, system::Commands};

use naia_bevy_server::Server;

use bevy_http_client::{HttpClient, ResponseError};
use bevy_http_server::HttpServer;
use config::SOCIAL_SERVER_GLOBAL_SECRET;
use logging::{info, warn};

use session_server_http_proto::{SocialPatchGlobalChatMessagesRequest, SocialPatchGlobalChatMessagesResponse, SocialPatchMatchLobbiesRequest, SocialPatchMatchLobbiesResponse, SocialPatchUsersRequest, SocialPatchUsersResponse};
use session_server_naia_proto::components::PublicUserInfo;

use crate::{social::SocialManager, user::UserManager};

pub fn recv_patch_users_request(
    mut commands: Commands,
    mut social_manager: ResMut<SocialManager>,
    mut http_server: ResMut<HttpServer>,
    mut http_client: ResMut<HttpClient>,
    mut user_manager: ResMut<UserManager>,
    mut naia_server: Server,
    mut users_q: Query<&mut PublicUserInfo>,
) {
    while let Some((_addr, request, response_key)) =
        http_server.receive::<SocialPatchUsersRequest>()
    {
        if request.social_secret() != SOCIAL_SERVER_GLOBAL_SECRET {
            warn!("invalid request secret");
            http_server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("received patch users request");

        social_manager.user_presence_manager.patch_users(
            &mut commands,
            &mut naia_server,
            &mut http_client,
            &mut user_manager,
            &mut users_q,
            request.user_patches(),
        );

        // responding
        http_server.respond(response_key, Ok(SocialPatchUsersResponse));
    }
}

pub fn recv_patch_global_chat_messages_request(
    mut commands: Commands,
    mut social_manager: ResMut<SocialManager>,
    mut http_server: ResMut<HttpServer>,
    mut http_client: ResMut<HttpClient>,
    mut user_manager: ResMut<UserManager>,
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

        info!("received patch global chat messages request");
        let user_presence_room_key = social_manager.user_presence_manager.room_key();

        social_manager.global_chat_manager.patch_global_chat_messages(
            &mut commands,
            &mut naia_server,
            &mut http_client,
            &mut user_manager,
            &user_presence_room_key,
            request.new_messages(),
        );

        // responding
        http_server.respond(response_key, Ok(SocialPatchGlobalChatMessagesResponse));
    }
}

pub fn recv_patch_match_lobby_request(
    mut commands: Commands,
    mut social_manager: ResMut<SocialManager>,
    mut http_server: ResMut<HttpServer>,
    mut http_client: ResMut<HttpClient>,
    mut user_manager: ResMut<UserManager>,
    mut naia_server: Server,
) {
    while let Some((_addr, request, response_key)) =
        http_server.receive::<SocialPatchMatchLobbiesRequest>()
    {
        if request.social_secret() != SOCIAL_SERVER_GLOBAL_SECRET {
            warn!("invalid request secret");
            http_server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("received patch match lobbies request");

        let user_presence_room_key = social_manager.user_presence_manager.room_key();
        social_manager.match_lobby_manager.patch_match_lobbies(
            &mut commands,
            &mut naia_server,
            &mut http_client,
            &mut user_manager,
            &user_presence_room_key,
            request.added_match_lobbies(),
            request.removed_match_lobbies(),
        );

        // responding
        http_server.respond(response_key, Ok(SocialPatchMatchLobbiesResponse));
    }
}