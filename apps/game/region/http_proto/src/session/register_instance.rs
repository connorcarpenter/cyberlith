use std::net::SocketAddr;
use naia_serde::{SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};
use serde::SerdeSocketAddr;

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SessionRegisterInstanceRequest {
    http_addr: SerdeSocketAddr,
    signal_addr: SerdeSocketAddr,
}

impl SessionRegisterInstanceRequest {
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
pub struct SessionRegisterInstanceResponse;

impl SessionRegisterInstanceResponse {
    pub fn new() -> Self {
        Self {

        }
    }
}

// Traits
impl ApiRequest for SessionRegisterInstanceRequest {
    type Response = SessionRegisterInstanceResponse;

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "session/register_instance"
    }
}

impl ApiResponse for SessionRegisterInstanceResponse {}