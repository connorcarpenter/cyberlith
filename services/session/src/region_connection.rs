use bevy_ecs::change_detection::ResMut;

use bevy_http_client::{HttpClient, ResponseError};
use bevy_http_server::HttpServer;
use config::{
    REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR, REGION_SERVER_SECRET,
    SESSION_SERVER_GLOBAL_SECRET, SESSION_SERVER_HTTP_PORT, SESSION_SERVER_RECV_ADDR,
};
use logging::{info, warn};

use region_server_http_proto::SessionRegisterInstanceRequest;
use session_server_http_proto::{HeartbeatRequest, HeartbeatResponse};

use crate::global::Global;

pub fn send_register_instance_request(
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    if global.connected() {
        return;
    }
    if global.waiting_for_registration_response() {
        return;
    }
    if !global.time_to_resend_registration() {
        return;
    }

    //info!("Sending request to register instance with region server ..");
    let request = SessionRegisterInstanceRequest::new(
        SESSION_SERVER_GLOBAL_SECRET,
        global.instance_secret(),
        SESSION_SERVER_RECV_ADDR,
        SESSION_SERVER_HTTP_PORT,
    );
    let key = http_client.send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request);

    global.set_register_instance_response_key(key);
    global.sent_to_region_server();
}

pub fn recv_register_instance_response(
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    if let Some(response_key) = global.register_instance_response_key() {
        if let Some(result) = http_client.recv(response_key) {
            match result {
                Ok(_response) => {
                    info!("received from regionserver: instance registered!");
                    global.set_connected();
                }
                Err(error) => {
                    warn!("error: {}", error.to_string());
                }
            }
            global.clear_register_instance_response_key();
        }
    }
}

pub fn recv_heartbeat_request(mut global: ResMut<Global>, mut server: ResMut<HttpServer>) {
    while let Some((_addr, request, response_key)) = server.receive::<HeartbeatRequest>() {
        if request.region_secret() != REGION_SERVER_SECRET {
            warn!("invalid request secret");
            server.respond(response_key, Err(ResponseError::Unauthenticated));
            continue;
        }

        info!("Heartbeat request received from region server");

        // setting last heard
        global.heard_from_region_server();

        // responding
        // info!("Sending heartbeat response to region server ..");
        server.respond(response_key, Ok(HeartbeatResponse));
    }
}

pub fn process_region_server_disconnect(mut global: ResMut<Global>) {
    if global.connected() {
        if global.time_to_disconnect() {
            info!("disconnecting from region server");
            global.set_disconnected();
        }
    }
}
