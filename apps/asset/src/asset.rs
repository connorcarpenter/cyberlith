
use log::info;

use http_client::ResponseError;
use http_server::Server;

use asset_server_http_proto::{AssetRequest, AssetResponse};

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

async fn async_impl(_: AssetRequest) -> Result<AssetResponse, ResponseError> {
    info!("Asset request received, sending response");
    Ok(AssetResponse)
}
