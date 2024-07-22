use bevy_ecs::change_detection::ResMut;

use bevy_http_client::{ApiRequest, ApiResponse, HttpClient, ResponseError};
use bevy_http_server::HttpServer;
use config::REGION_SERVER_SECRET;
use logging::warn;
use region_server_http_proto::WorldRegisterInstanceResponse;
use world_server_http_proto::{HeartbeatRequest, HeartbeatResponse};

use crate::region::RegionManager;

pub fn recv_heartbeat_request(
    mut region_manager: ResMut<RegionManager>,
    mut server: ResMut<HttpServer>,
) {
    while let Some((_addr, request, response_key)) = server.receive::<HeartbeatRequest>() {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        let host = "world";
        let remote = "region";
        bevy_http_client::log_util::recv_req(host, remote, HeartbeatRequest::name());

        // info!("Heartbeat request received from region server");

        // setting last heard
        region_manager.heard_from_region_server();

        // responding
        // info!("Sending heartbeat response to region server ..");
        bevy_http_client::log_util::send_res(host, HeartbeatResponse::name());
        server.respond(response_key, Ok(HeartbeatResponse));
    }
}

pub fn recv_register_instance_response(
    mut http_client: ResMut<HttpClient>,
    mut region_manager: ResMut<RegionManager>,
) {
    if let Some(response_key) = region_manager.register_instance_response_key() {
        if let Some(result) = http_client.recv(response_key) {
            let host = "world";
            let remote = "region";
            bevy_http_client::log_util::recv_res(
                host,
                remote,
                WorldRegisterInstanceResponse::name(),
            );

            match result {
                Ok(_response) => {
                    // info!("received from regionserver: instance registered!");
                    region_manager.set_connected();
                }
                Err(error) => {
                    warn!("error: {}", error.to_string());
                }
            }
            region_manager.clear_register_instance_response_key();
        }
    }
}