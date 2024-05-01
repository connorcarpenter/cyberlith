
use http_server::ResponseError;

use gateway_http_proto::{UserRegisterRequest as GatewayUserRegisterRequest, UserRegisterResponse as GatewayUserRegisterResponse};

async fn handler(
    _incoming_request: GatewayUserRegisterRequest,
) -> Result<GatewayUserRegisterResponse, ResponseError> {
    todo!()
}