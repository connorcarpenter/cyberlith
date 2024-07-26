use auth_server_types::UserId;
use config::{REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR, SOCIAL_SERVER_GLOBAL_SECRET};
use http_client::{HttpClient, ResponseError};
use http_server::{ApiRequest, ApiResponse};
use region_server_http_proto::{WorldConnectRequest, WorldConnectResponse};
use social_server_types::LobbyId;

use crate::region::RegionServerState;

pub async fn send_world_connect_request(
    region_server_state: &mut RegionServerState,
    lobby_id: LobbyId,
    // Vec<session_instance_secret, Vec<UserId>>
    user_ids: Vec<(String, Vec<UserId>)>,
) -> Result<(String, Vec<(UserId, String)>), ResponseError> {
    if !region_server_state.connected() {
        return Err(ResponseError::NetworkError(
            "region server not connected".to_string(),
        ));
    }

    let request = WorldConnectRequest::new(SOCIAL_SERVER_GLOBAL_SECRET, lobby_id, user_ids);

    let host = "social";
    let remote = "region";
    http_server::log_util::send_req(host, remote, WorldConnectRequest::name());
    let response_result =
        HttpClient::send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request).await;
    http_server::log_util::recv_res(host, remote, WorldConnectResponse::name());
    region_server_state.sent_to_region_server();

    match response_result {
        Ok(response) => {
            region_server_state.heard_from_region_server();

            let WorldConnectResponse {
                world_server_instance_secret,
                login_tokens,
            } = response;

            return Ok((world_server_instance_secret, login_tokens));
        }
        Err(err) => {
            return Err(err);
        }
    }
}
