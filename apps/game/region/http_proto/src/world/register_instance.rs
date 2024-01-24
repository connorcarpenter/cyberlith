use std::net::SocketAddr;

use naia_serde::{SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};
use serde::SerdeSocketAddr;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldRegisterInstanceRequest {
    http_addr: SerdeSocketAddr,
    signal_addr: SerdeSocketAddr,
}

impl WorldRegisterInstanceRequest {
    pub fn new(http_addr: SocketAddr, signal_addr: SocketAddr) -> Self {
        Self {
            http_addr: SerdeSocketAddr::new(http_addr),
            signal_addr: SerdeSocketAddr::new(signal_addr),
        }
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
