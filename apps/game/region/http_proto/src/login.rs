use std::net::SocketAddr;

use naia_serde::{BitWrite, SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};
use serde::SerdeSocketAddr;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

impl LoginRequest {
    pub fn new(username: &str, password: &str) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct LoginResponse {
    pub session_server_addr: SerdeSocketAddr,
}

impl LoginResponse {
    pub fn new(session_server_addr: SocketAddr) -> Self {
        Self {
            session_server_addr: SerdeSocketAddr::new(session_server_addr),
        }
    }
}

// Traits
impl ApiRequest for LoginRequest {
    type Response = LoginResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "login"
    }
}

impl ApiResponse for LoginResponse {}
