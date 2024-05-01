
use http_server::ResponseError;

use gateway_http_proto::{UserLoginRequest as GatewayUserLoginRequest, UserLoginResponse as GatewayUserLoginResponse};

async fn handler(
    _incoming_request: GatewayUserLoginRequest,
) -> Result<GatewayUserLoginResponse, ResponseError> {
    todo!()
}