
use http_server::ResponseError;

use gateway_http_proto::{UserPasswordResetRequest as GatewayUserPasswordResetRequest, UserPasswordResetResponse as GatewayUserPasswordResetResponse};

async fn handler(
    _incoming_request: GatewayUserPasswordResetRequest,
) -> Result<GatewayUserPasswordResetResponse, ResponseError> {
    todo!()
}