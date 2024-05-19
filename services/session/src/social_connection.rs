use bevy_ecs::system::ResMut;

use bevy_http_client::ResponseError;
use bevy_http_server::HttpServer;

use config::REGION_SERVER_SECRET;
use logging::{info, warn};

use session_server_http_proto::{
    ConnectSocialServerRequest, ConnectSocialServerResponse, DisconnectSocialServerRequest,
    DisconnectSocialServerResponse,
};

use crate::{global::Global, region::RegionConnection};

pub fn recv_connect_social_server_request(
    mut global: ResMut<Global>,
    mut region: ResMut<RegionConnection>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) = server.receive::<ConnectSocialServerRequest>()
    {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("Connect Social Server request received from region server");

        // setting last heard
        region.heard_from_region_server();

        // store social server details
        global.set_social_server(request.http_addr(), request.http_port());

        // responding
        // info!("Sending connect social server response to region server ..");
        server.respond(response_key, Ok(ConnectSocialServerResponse));
    }
}

pub fn recv_disconnect_social_server_request(
    mut global: ResMut<Global>,
    mut region: ResMut<RegionConnection>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) =
        server.receive::<DisconnectSocialServerRequest>()
    {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("Disconnect Social Server request received from region server");

        // setting last heard
        region.heard_from_region_server();

        // erase social server details
        global.clear_social_server();

        // responding
        // info!("Sending connect social server response to region server ..");
        server.respond(response_key, Ok(DisconnectSocialServerResponse));
    }
}
