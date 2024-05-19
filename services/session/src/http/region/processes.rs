use bevy_ecs::{system::Res, change_detection::ResMut};

use bevy_http_client::HttpClient;
use config::{
    REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR,
    SESSION_SERVER_GLOBAL_SECRET, SESSION_SERVER_HTTP_PORT, SESSION_SERVER_RECV_ADDR,
};
use logging::info;

use region_server_http_proto::{SessionRegisterInstanceRequest, WorldConnectRequest};

use crate::{session_instance::SessionInstance, user_manager::UserManager, http::{region::RegionConnection, world::WorldConnections}};

pub fn send_register_instance_request(
    session_instance: Res<SessionInstance>,
    mut region: ResMut<RegionConnection>,
    mut http_client: ResMut<HttpClient>,
) {
    if region.connected() {
        return;
    }
    if region.waiting_for_registration_response() {
        return;
    }
    if !region.time_to_resend_registration() {
        return;
    }

    //info!("Sending request to register instance with region server ..");
    let request = SessionRegisterInstanceRequest::new(
        SESSION_SERVER_GLOBAL_SECRET,
        session_instance.instance_secret(),
        SESSION_SERVER_RECV_ADDR,
        SESSION_SERVER_HTTP_PORT,
    );
    let key = http_client.send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request);

    region.set_register_instance_response_key(key);
    region.sent_to_region_server();
}

pub fn process_region_server_disconnect(
    mut region: ResMut<RegionConnection>,
) {
    if region.connected() {
        if region.time_to_disconnect() {
            info!("disconnecting from region server");
            region.set_disconnected();
        }
    }
}

pub fn send_world_connect_requests(
    session_instance: Res<SessionInstance>,
    mut http_client: ResMut<HttpClient>,
    mut user_manager: ResMut<UserManager>,
    mut world_connections: ResMut<WorldConnections>,
) {
    let worldless_users = user_manager.get_users_ready_to_connect_to_world(world_connections.world_connect_resend_rate());
    for (user_key, user_id) in worldless_users {
        let request = WorldConnectRequest::new(session_instance.instance_secret(), user_id);
        let key = http_client.send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request);
        world_connections.add_world_connect_response_key(&user_key, key);
    }
}