use bevy_ecs::change_detection::ResMut;

use bevy_http_client::{HttpClient, ResponseError};
use bevy_http_server::HttpServer;
use config::REGION_SERVER_SECRET;
use logging::{info, warn};

use session_server_http_proto::{HeartbeatRequest, HeartbeatResponse};

use crate::region::RegionConnection;

pub fn recv_register_instance_response(
    mut http_client: ResMut<HttpClient>,
    mut region: ResMut<RegionConnection>,
) {
    if let Some(response_key) = region.register_instance_response_key() {
        if let Some(result) = http_client.recv(response_key) {
            match result {
                Ok(_response) => {
                    info!("received from regionserver: instance registered!");
                    region.set_connected();
                }
                Err(error) => {
                    warn!("error: {}", error.to_string());
                }
            }
            region.clear_register_instance_response_key();
        }
    }
}

pub fn recv_heartbeat_request(
    mut region: ResMut<RegionConnection>,
    mut server: ResMut<HttpServer>
) {
    while let Some((_addr, request, response_key)) = server.receive::<HeartbeatRequest>() {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("Heartbeat request received from region server");

        // setting last heard
        region.heard_from_region_server();

        // responding
        // info!("Sending heartbeat response to region server ..");
        server.respond(response_key, Ok(HeartbeatResponse));
    }
}