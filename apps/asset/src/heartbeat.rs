
use log::info;

use http_client::ResponseError;
use http_server::Server;

use asset_server_http_proto::{HeartbeatRequest, HeartbeatResponse};

pub fn endpoint(
    server: &mut Server,
) {
    server.endpoint(
        move |(_addr, req)| {
            async move {
                async_impl(req).await
            }
        }
    );
}

async fn async_impl(_: HeartbeatRequest) -> Result<HeartbeatResponse, ResponseError> {
    info!("Heartbeat request received from region server, sending response");
    Ok(HeartbeatResponse)
}
