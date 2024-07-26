use bevy_ecs::{
    change_detection::ResMut,
    prelude::Query,
    system::{Commands, Res},
};

use naia_bevy_server::Server;

use auth_server_types::UserId;
use bevy_http_client::{HttpClient, ResponseError};
use bevy_http_server::HttpServer;
use config::SOCIAL_SERVER_GLOBAL_SECRET;
use logging::{info, warn};
use session_server_http_proto::{
    SocialPatchGlobalChatMessagesRequest, SocialPatchGlobalChatMessagesResponse,
    SocialPatchMatchLobbiesRequest, SocialPatchMatchLobbiesResponse, SocialPatchUsersRequest,
    SocialPatchUsersResponse, SocialWorldConnectRequest, SocialWorldConnectResponse,
};
use session_server_naia_proto::{
    channels::PrimaryChannel,
    components::{Lobby, User},
    messages::WorldConnectToken,
};
use social_server_types::LobbyId;

use crate::{social::SocialManager, user::UserManager, world::WorldManager};

pub fn recv_patch_users_request(
    mut commands: Commands,
    mut social_manager: ResMut<SocialManager>,
    mut http_server: ResMut<HttpServer>,
    mut http_client: ResMut<HttpClient>,
    mut user_manager: ResMut<UserManager>,
    mut naia_server: Server,
    mut users_q: Query<&mut User>,
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
        let main_menu_room_key = social_manager.global_room_key().unwrap();
        social_manager.user_presence_manager.patch_users(
            &mut commands,
            &mut naia_server,
            &mut http_client,
            &mut user_manager,
            &mut users_q,
            request.user_patches(),
            &main_menu_room_key,
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
        let main_menu_room_key = social_manager.global_room_key().unwrap();

        social_manager
            .chat_message_manager
            .patch_global_chat_messages(
                &mut commands,
                &mut naia_server,
                &mut http_client,
                &mut user_manager,
                &main_menu_room_key,
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
    mut lobby_q: Query<&mut Lobby>,
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

        let main_menu_room_key = social_manager.global_room_key().unwrap();
        social_manager.patch_match_lobbies(
            &mut commands,
            &mut naia_server,
            &mut http_client,
            &mut user_manager,
            &mut lobby_q,
            &main_menu_room_key,
            request.patches(),
        );

        // responding
        http_server.respond(response_key, Ok(SocialPatchMatchLobbiesResponse));
    }
}

pub fn recv_world_connect(
    mut naia_server: Server,
    mut http_server: ResMut<HttpServer>,
    mut user_manager: ResMut<UserManager>,
    mut world_manager: ResMut<WorldManager>,
    social_manager: Res<SocialManager>,
) {
    while let Some((_addr, request, response_key)) =
        http_server.receive::<SocialWorldConnectRequest>()
    {
        if request.social_secret() != SOCIAL_SERVER_GLOBAL_SECRET {
            warn!("invalid request secret");
            http_server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("received world connect request");

        let world_instance_secret = request.world_instance_secret().to_string();
        let lobby_id = request.lobby_id();
        let login_tokens = request.login_tokens().clone();

        process_world_connect(
            &mut naia_server,
            &mut user_manager,
            &mut world_manager,
            &social_manager,
            &world_instance_secret,
            &lobby_id,
            login_tokens,
        );

        // responding
        http_server.respond(response_key, Ok(SocialWorldConnectResponse));
    }
}

fn process_world_connect(
    naia_server: &mut Server,
    user_manager: &mut UserManager,
    world_manager: &mut WorldManager,
    social_manager: &SocialManager,
    world_instance_secret: &str,
    lobby_id: &LobbyId,
    login_tokens: Vec<(UserId, String)>,
) {
    let global_room_key = social_manager.global_room_key().unwrap();
    let lobby_room_key = social_manager
        .lobby_manager
        .get_lobby_room_key(&lobby_id)
        .unwrap();
    let lobby_entity = social_manager
        .lobby_manager
        .get_lobby_entity(&lobby_id)
        .unwrap();

    // move lobby entity from global room to lobby room
    naia_server
        .room_mut(&global_room_key)
        .remove_entity(&lobby_entity);
    naia_server
        .room_mut(&lobby_room_key)
        .add_entity(&lobby_entity);

    // handle user login tokens
    for (user_id, login_token) in login_tokens {
        let user_key = user_manager.user_id_to_key(&user_id).unwrap();

        // store world instance secret with user key
        user_manager.user_set_world_connected(&user_key, &world_instance_secret);
        world_manager.world_set_user_connected(&user_id, &user_key, &world_instance_secret);

        let user_entity = user_manager.get_user_entity(&user_id).unwrap();

        naia_server
            .room_mut(&global_room_key)
            // remove user entity from global room
            .remove_entity(&user_entity)
            // remove user from global room
            .remove_user(&user_key);

        naia_server
            .room_mut(&lobby_room_key)
            .add_entity(&user_entity);

        // send world connect token to user
        // info!("sending world connect token to user");
        let token = WorldConnectToken::new(&login_token);
        naia_server.send_message::<PrimaryChannel, WorldConnectToken>(&user_key, &token);
    }
}
