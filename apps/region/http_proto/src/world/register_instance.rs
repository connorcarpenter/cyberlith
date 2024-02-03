
use naia_serde::{SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldRegisterInstanceRequest {
    world_secret: String,
    http_addr: String,
    http_port: u16,
    public_url: String,
}

impl WorldRegisterInstanceRequest {
    pub fn new(world_secret: &str, http_addr: &str, http_port: u16, public_url: &str) -> Self {
        Self {
            world_secret: world_secret.to_string(),
            http_addr: http_addr.to_string(),
            http_port,
            public_url: public_url.to_string(),
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

    pub fn public_url(&self) -> String {
        self.public_url.clone()
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
