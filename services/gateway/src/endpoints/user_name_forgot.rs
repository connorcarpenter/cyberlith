
use http_server::ResponseError;

use gateway_http_proto::{UserNameForgotRequest as GatewayUserNameForgotRequest, UserNameForgotResponse as GatewayUserNameForgotResponse};

async fn handler(
    _incoming_request: GatewayUserNameForgotRequest,
) -> Result<GatewayUserNameForgotResponse, ResponseError> {
    todo!()
}