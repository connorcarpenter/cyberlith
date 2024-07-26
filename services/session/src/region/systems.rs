use bevy_ecs::{change_detection::ResMut, system::Res};

use bevy_http_client::{log_util, ApiRequest, HttpClient};
use config::{
    REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR, SESSION_SERVER_GLOBAL_SECRET,
    SESSION_SERVER_HTTP_PORT, SESSION_SERVER_RECV_ADDR,
};
use logging::info;

use region_server_http_proto::SessionRegisterInstanceRequest;

use crate::{
    asset::asset_manager::AssetManager, region::RegionManager, session_instance::SessionInstance,
    social::SocialManager,
};

pub fn send_register_instance_request(
    session_instance: Res<SessionInstance>,
    mut region: ResMut<RegionManager>,
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

    let host = "session";
    let remote = "region";
    log_util::send_req(host, remote, SessionRegisterInstanceRequest::name());
    let key = http_client.send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request);

    region.set_register_instance_response_key(key);
    region.sent_to_region_server();
}

pub fn process_region_server_disconnect(
    mut region: ResMut<RegionManager>,
    mut asset: ResMut<AssetManager>,
    mut social: ResMut<SocialManager>,
) {
    if region.connected() {
        if region.time_to_disconnect() {
            info!("disconnecting from region server");
            region.set_disconnected();

            // disconnect from asset server
            asset.clear_asset_server();

            // disconnect from social server
            social.clear_social_server();

            // TODO: disconnect from world servers
        }
    }
}
