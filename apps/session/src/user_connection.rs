
use bevy_ecs::change_detection::ResMut;
use bevy_log::{info, warn};

use bevy_http_client::ResponseError;
use bevy_http_server::HttpServer;

use session_server_http_proto::{IncomingUserRequest, IncomingUserResponse};
use config::REGION_SERVER_SECRET;

use crate::global::Global;

pub fn recv_login_request(
    mut global: ResMut<Global>,
    mut server: ResMut<HttpServer>
) {
    while let Some((_addr, request, response_key)) = server.receive::<IncomingUserRequest>() {

        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("Login request received from region server: Login(token: {})", request.login_token);

        global.add_login_token(&request.login_token);

        info!("Sending login response to region server ..");

        server.respond(response_key, Ok(IncomingUserResponse));
    }
}