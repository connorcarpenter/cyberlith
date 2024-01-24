use std::net::SocketAddr;

use naia_serde::{SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};
use serde::SerdeSocketAddr;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SessionUserLoginRequest {
    pub username: String,
    pub password: String,
}

impl SessionUserLoginRequest {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct SessionUserLoginResponse {
    pub session_server_addr: SerdeSocketAddr,
    pub token: String,
}

impl SessionUserLoginResponse {
    pub fn new(session_server_addr: SocketAddr, token: &str) -> Self {
        Self {
            session_server_addr: SerdeSocketAddr::new(session_server_addr),
            token: token.to_string(),
        }
    }
}

// Traits
impl ApiRequest for SessionUserLoginRequest {
    type Response = SessionUserLoginResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "session/user_login"
    }
}

impl ApiResponse for SessionUserLoginResponse {}
