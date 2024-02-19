use bevy_ecs::system::ResMut;
use bevy_log::{info, warn};

use bevy_http_client::ResponseError;
use bevy_http_server::HttpServer;

use session_server_http_proto::{ConnectAssetServerRequest, ConnectAssetServerResponse, DisconnectAssetServerRequest, DisconnectAssetServerResponse};
use config::REGION_SERVER_SECRET;

use crate::global::Global;

pub fn recv_connect_asset_server_request(
    mut global: ResMut<Global>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) = server.receive::<ConnectAssetServerRequest>() {

        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("Connect Asset Server request received from region server");

        // setting last heard
        global.heard_from_region_server();

        // store asset server details
        global.set_asset_server(request.http_addr(), request.http_port());

        // responding
        // info!("Sending connect asset server response to region server ..");
        server.respond(response_key, Ok(ConnectAssetServerResponse));
    }
}

pub fn recv_disconnect_asset_server_request(
    mut global: ResMut<Global>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) = server.receive::<DisconnectAssetServerRequest>() {

        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("Disconnect Asset Server request received from region server");

        // setting last heard
        global.heard_from_region_server();

        // erase asset server details
        global.clear_asset_server();

        // responding
        // info!("Sending connect asset server response to region server ..");
        server.respond(response_key, Ok(DisconnectAssetServerResponse));
    }
}