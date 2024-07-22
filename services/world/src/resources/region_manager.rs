use std::time::{Duration, Instant};

use bevy_ecs::{system::{Res, Resource}, change_detection::ResMut};

use bevy_http_client::{
    ApiRequest, ApiResponse, HttpClient, ResponseError, ResponseKey as ClientResponseKey,
};
use bevy_http_server::HttpServer;
use config::{
    REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR, REGION_SERVER_SECRET, WORLD_SERVER_GLOBAL_SECRET,
    WORLD_SERVER_HTTP_PORT, WORLD_SERVER_RECV_ADDR,
};
use logging::{info, warn};

use region_server_http_proto::{WorldRegisterInstanceRequest, WorldRegisterInstanceResponse};
use world_server_http_proto::{HeartbeatRequest, HeartbeatResponse};

use crate::resources::{world_instance::WorldInstance, user_manager::UserManager};

pub enum ConnectionState {
    Disconnected,
    Connected,
}

#[derive(Resource)]
pub struct RegionManager {
    region_server_connection_state: ConnectionState,
    region_server_last_sent: Instant,
    region_server_last_heard: Instant,
    register_instance_response_key: Option<ClientResponseKey<WorldRegisterInstanceResponse>>,
    registration_resend_rate: Duration,
    region_server_disconnect_timeout: Duration,
}

impl RegionManager {
    pub fn new(
        registration_resend_rate: Duration,
        region_server_disconnect_timeout: Duration,
    ) -> Self {
        Self {
            region_server_connection_state: ConnectionState::Disconnected,
            region_server_last_sent: Instant::now(),
            region_server_last_heard: Instant::now(),
            register_instance_response_key: None,
            registration_resend_rate,
            region_server_disconnect_timeout,
        }
    }

    pub fn register_instance_response_key(
        &self,
    ) -> Option<&ClientResponseKey<WorldRegisterInstanceResponse>> {
        self.register_instance_response_key.as_ref()
    }

    pub fn set_register_instance_response_key(
        &mut self,
        response_key: ClientResponseKey<WorldRegisterInstanceResponse>,
    ) {
        self.register_instance_response_key = Some(response_key);
    }

    pub fn clear_register_instance_response_key(&mut self) {
        self.register_instance_response_key = None;
    }

    pub fn waiting_for_registration_response(&self) -> bool {
        self.register_instance_response_key.is_some()
    }

    pub fn time_to_resend_registration(&self) -> bool {
        let time_since_last_sent = self.region_server_last_sent.elapsed();
        time_since_last_sent >= self.registration_resend_rate
    }

    pub fn time_to_disconnect(&self) -> bool {
        let time_since_last_heard = self.region_server_last_heard.elapsed();
        time_since_last_heard >= self.region_server_disconnect_timeout
    }

    pub fn heard_from_region_server(&mut self) {
        self.region_server_last_heard = Instant::now();
    }

    pub fn sent_to_region_server(&mut self) {
        self.region_server_last_sent = Instant::now();
    }

    pub fn connected(&self) -> bool {
        match self.region_server_connection_state {
            ConnectionState::Connected => true,
            ConnectionState::Disconnected => false,
        }
    }

    pub fn set_connected(&mut self) {
        self.region_server_connection_state = ConnectionState::Connected;
        self.heard_from_region_server();
    }

    pub fn disconnect_region_server(&mut self) {
        self.region_server_connection_state = ConnectionState::Disconnected;
    }
}

pub fn send_register_instance_request(
    mut http_client: ResMut<HttpClient>,
    world_instance: Res<WorldInstance>,
    mut region_manager: ResMut<RegionManager>,
) {
    if region_manager.connected() {
        return;
    }
    if region_manager.waiting_for_registration_response() {
        return;
    }
    if !region_manager.time_to_resend_registration() {
        return;
    }

    //info!("Sending request to register instance with region server ..");
    let request = WorldRegisterInstanceRequest::new(
        WORLD_SERVER_GLOBAL_SECRET,
        world_instance.instance_secret(),
        WORLD_SERVER_RECV_ADDR,
        WORLD_SERVER_HTTP_PORT,
    );
    let key = http_client.send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request);

    region_manager.set_register_instance_response_key(key);
    region_manager.sent_to_region_server();
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

pub fn process_region_server_disconnect(
    mut user_manager: ResMut<UserManager>,
    mut region_manager: ResMut<RegionManager>,
) {
    if region_manager.connected() {
        if region_manager.time_to_disconnect() {
            info!("disconnecting from region server");
            region_manager.disconnect_region_server();
            user_manager.disconnect_region_server();
        }
    }
}
