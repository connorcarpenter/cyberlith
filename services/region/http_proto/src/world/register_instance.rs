use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct WorldRegisterInstanceRequest {
    global_secret: String,
    instance_secret: String,
    http_addr: String,
    http_port: u16,
}

impl WorldRegisterInstanceRequest {
    pub fn new(
        global_secret: &str,
        instance_secret: &str,
        http_addr: &str,
        http_port: u16,
    ) -> Self {
        Self {
            global_secret: global_secret.to_string(),
            instance_secret: instance_secret.to_string(),
            http_addr: http_addr.to_string(),
            http_port,
        }
    }

    pub fn global_secret(&self) -> &str {
        &self.global_secret
    }

    pub fn instance_secret(&self) -> &str {
        &self.instance_secret
    }

    pub fn http_addr(&self) -> &str {
        &self.http_addr
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct WorldRegisterInstanceResponse;

impl WorldRegisterInstanceResponse {
    pub fn new() -> Self {
        Self {}
    }
}

// Traits
impl ApiRequest for WorldRegisterInstanceRequest {
    type Response = WorldRegisterInstanceResponse;

    fn name() -> &'static str {
        "WorldRegisterInstanceRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "world/register_instance"
    }
}

impl ApiResponse for WorldRegisterInstanceResponse {
    fn name() -> &'static str {
        "WorldRegisterInstanceResponse"
    }
}
