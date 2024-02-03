
use naia_serde::{SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldRegisterInstanceRequest {
    world_secret: String,
    http_addr: String,
    http_port: u16,
    signal_addr: String,
    signal_port: u16,
}

impl WorldRegisterInstanceRequest {
    pub fn new(world_secret: &str, http_addr: &str, http_port: u16, signal_addr: &str, signal_port: u16) -> Self {
        Self {
            world_secret: world_secret.to_string(),
            http_addr: http_addr.to_string(),
            http_port,
            signal_addr: signal_addr.to_string(),
            signal_port,
        }
    }

    pub fn world_secret(&self) -> &str {
        &self.world_secret
    }

    pub fn http_addr(&self) -> String {
        self.http_addr.clone()
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    pub fn signal_addr(&self) -> String {
        self.signal_addr.clone()
    }

    pub fn signal_port(&self) -> u16 {
        self.signal_port
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
