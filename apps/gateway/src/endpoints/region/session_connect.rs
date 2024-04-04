use log::{info, warn};

use http_client::{HttpClient, ResponseError};
use http_server::Server;

use config::{GATEWAY_SECRET, REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR};
use gateway_http_proto::{SessionConnectRequest as GatewaySessionConnectRequest, SessionConnectResponse as GatewaySessionConnectResponse};
use region_server_http_proto::SessionConnectRequest as RegionSessionConnectRequest;

pub fn session_connect(server: &mut Server) {
    server.endpoint(move |(_addr, req)| async move { async_impl(req).await });
}

async fn async_impl(incoming_request: GatewaySessionConnectRequest) -> Result<GatewaySessionConnectResponse, ResponseError> {
    info!("session_connect request <- client");

    info!("session_connect request -> region server");

    let region_request = RegionSessionConnectRequest::new(
        GATEWAY_SECRET,
        &incoming_request.username,
        &incoming_request.password,
    );
    let Ok(region_response) =
        HttpClient::send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, region_request).await
    else {
        warn!("FAILED session_connect request -> region server");
        return Err(ResponseError::InternalServerError(
            "FAILED session_connect request -> region server".to_string(),
        ));
    };

    info!(
        "session_connect response <- region server: (webrtc_url: {:?}, token: {})",
        region_response.session_server_public_webrtc_url, region_response.token,
    );

    info!("session_connect response -> client");

    Ok(GatewaySessionConnectResponse::new(
        &region_response.session_server_public_webrtc_url,
        &region_response.token,
    ))
}
