use auth_server_types::UserId;
use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

use crate::AccessToken;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct AccessTokenValidateRequest {
    pub access_token: AccessToken,
}

impl AccessTokenValidateRequest {
    pub fn new(access_token: AccessToken) -> Self {
        Self { access_token }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct AccessTokenValidateResponse {
    pub user_id: UserId,
    pub user_name: String,
}

impl AccessTokenValidateResponse {
    pub fn new(user_id: UserId, user_name: String) -> Self {
        Self { user_id, user_name }
    }
}

// Traits
impl ApiRequest for AccessTokenValidateRequest {
    type Response = AccessTokenValidateResponse;

    fn name() -> &'static str {
        "AccessTokenValidateRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "access_token_validate"
    }
}

impl ApiResponse for AccessTokenValidateResponse {
    fn name() -> &'static str {
        "AccessTokenValidateResponse"
    }
}
