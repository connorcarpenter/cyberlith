use std::net::SocketAddr;

use naia_serde::{SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};
use serde::SerdeSocketAddr;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldUserLoginRequest {
    session_secret: String,
}

impl WorldUserLoginRequest {
    pub fn new(session_secret: &str) -> Self {
        Self {
            session_secret: session_secret.to_string(),
        }
    }
}

// Response
#[derive(Serde, PartialEq, Clone, Eq, Hash)]
pub struct WorldUserLoginResponse {
    pub world_server_addr: SerdeSocketAddr,
    pub token: String,
}

impl WorldUserLoginResponse {
    pub fn new(world_server_addr: SocketAddr, token: &str) -> Self {
        Self {
            world_server_addr: SerdeSocketAddr::new(world_server_addr),
            token: token.to_string(),
        }
    }
}

// Traits
impl ApiRequest for WorldUserLoginRequest {
    type Response = WorldUserLoginResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "world/user_login"
    }
}

impl ApiResponse for WorldUserLoginResponse {}
