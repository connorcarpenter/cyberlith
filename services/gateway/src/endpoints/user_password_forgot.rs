
use http_server::ResponseError;

use gateway_http_proto::{UserPasswordForgotRequest as GatewayUserPasswordForgotRequest, UserPasswordForgotResponse as GatewayUserPasswordForgotResponse};

async fn handler(
    _incoming_request: GatewayUserPasswordForgotRequest,
) -> Result<GatewayUserPasswordForgotResponse, ResponseError> {
    todo!()
}