use std::net::SocketAddr;

use naia_serde::{SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};
use serde::SerdeSocketAddr;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldRegisterInstanceRequest {
    world_secret: String,
    http_addr: SerdeSocketAddr,
    signal_addr: SerdeSocketAddr,
}

impl WorldRegisterInstanceRequest {
    pub fn new(world_secret: &str, http_addr: SocketAddr, signal_addr: SocketAddr) -> Self {
        Self {
            world_secret: world_secret.to_string(),
            http_addr: SerdeSocketAddr::new(http_addr),
            signal_addr: SerdeSocketAddr::new(signal_addr),
        }
    }

    pub fn world_secret(&self) -> &str {
        &self.world_secret
    }

    pub fn http_addr(&self) -> SocketAddr {
        self.http_addr.inner()
    }

    pub fn signal_addr(&self) -> SocketAddr {
        self.signal_addr.inner()
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct WorldRegisterInstanceResponse;

impl WorldRegisterInstanceResponse {
    pub fn new() -> Self {
        Self {

        }
    }
}

// Traits
impl ApiRequest for WorldRegisterInstanceRequest {
    type Response = WorldRegisterInstanceResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "world/register_instance"
    }
}

impl ApiResponse for WorldRegisterInstanceResponse {}
