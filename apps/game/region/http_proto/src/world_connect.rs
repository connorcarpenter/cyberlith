use std::net::SocketAddr;

use naia_serde::{SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};
use serde::SerdeSocketAddr;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldConnectRequest;

impl WorldConnectRequest {
    pub fn new() -> Self {
        Self
    }
}

// Response
#[derive(Serde, PartialEq, Clone, Eq, Hash)]
pub struct WorldConnectResponse {
    pub world_server_addr: SerdeSocketAddr,
    pub token: String,
}

impl WorldConnectResponse {
    pub fn new(world_server_addr: SocketAddr, token: &str) -> Self {
        Self {
            world_server_addr: SerdeSocketAddr::new(world_server_addr),
            token: token.to_string(),
        }
    }
}

// Traits
impl ApiRequest for WorldConnectRequest {
    type Response = WorldConnectResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "world"
    }
}

impl ApiResponse for WorldConnectResponse {}
