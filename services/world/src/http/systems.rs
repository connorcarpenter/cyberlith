use std::net::SocketAddr;

use bevy_ecs::change_detection::ResMut;

use naia_bevy_server::Server;

use bevy_http_client::ResponseError;
use bevy_http_server::HttpServer;
use config::{REGION_SERVER_SECRET, SELF_BINDING_ADDR, WORLD_SERVER_HTTP_PORT};
use logging::{info, warn};
use world_server_http_proto::{WorldConnectRequest, WorldConnectResponse};

use crate::{social::LobbyManager, user::UserManager};

pub fn init(mut server: ResMut<HttpServer>) {
    info!("World HTTP Server starting up");

    let socket_addr = SocketAddr::new(SELF_BINDING_ADDR.parse().unwrap(), WORLD_SERVER_HTTP_PORT);
    server.listen(socket_addr);
}

pub fn recv_world_connect_request(
    mut lobby_manager: ResMut<LobbyManager>,
    mut user_manager: ResMut<UserManager>,
    mut http_server: ResMut<HttpServer>,
    mut naia_server: Server,
) {
    while let Some((_addr, request, response_key)) = http_server.receive::<WorldConnectRequest>() {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            http_server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!(
            "Login request received from region server: (lobby_id: {:?}, tokens: {:?})",
            request.lobby_id(),
            request.login_tokens()
        );

        user_manager.recv_login_token(&request.lobby_id(), request.login_tokens());
        let lobby_room_key = naia_server.make_room().key();
        lobby_manager.insert_lobby_room_key(request.lobby_id(), lobby_room_key);

        info!("Sending login response to region server ..");

        http_server.respond(response_key, Ok(WorldConnectResponse::new()));
    }
}
