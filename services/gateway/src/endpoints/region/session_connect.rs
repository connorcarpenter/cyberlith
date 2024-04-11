use log::info;

use http_client::{HttpClient, ResponseError};
use http_server::{http_log_util, Server};

use config::{GATEWAY_SECRET, REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR};
use gateway_http_proto::{
    SessionConnectRequest as GatewaySessionConnectRequest,
    SessionConnectResponse as GatewaySessionConnectResponse,
};
use region_server_http_proto::SessionConnectRequest as RegionSessionConnectRequest;

pub fn session_connect(server: &mut Server) {
    server.endpoint(move |(_addr, req)| async move { async_impl(req).await });
}

async fn async_impl(
    incoming_request: GatewaySessionConnectRequest,
) -> Result<GatewaySessionConnectResponse, ResponseError> {
    http_log_util::recv_req("gateway", "client", "session_connect");

    http_log_util::send_req("gateway", "region_server", "session_connect");
    let region_request = RegionSessionConnectRequest::new(
        GATEWAY_SECRET,
        &incoming_request.username,
        &incoming_request.password,
    );
    let Ok(region_response) =
        HttpClient::send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, region_request).await
    else {
        return http_log_util::fail_recv_res("gateway", "region_server", "session_connect");
    };

    http_log_util::recv_res("gateway", "region_server", "session_connect");
    info!(
        "[webrtc_url: {:?}, token: {}]",
        region_response.session_server_public_webrtc_url, region_response.token,
    );

    http_log_util::send_res("gateway", "client", "session_connect");
    Ok(GatewaySessionConnectResponse::new(
        &region_response.session_server_public_webrtc_url,
        &region_response.token,
    ))
}
