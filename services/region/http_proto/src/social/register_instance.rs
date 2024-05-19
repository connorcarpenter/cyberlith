use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SocialRegisterInstanceRequest {
    global_secret: String,
    http_addr: String,
    http_port: u16,
}

impl SocialRegisterInstanceRequest {
    pub fn new(global_secret: &str, http_addr: &str, http_port: u16) -> Self {
        Self {
            global_secret: global_secret.to_string(),
            http_addr: http_addr.to_string(),
            http_port,
        }
    }

    pub fn global_secret(&self) -> &str {
        &self.global_secret
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
pub struct SocialRegisterInstanceResponse;

impl SocialRegisterInstanceResponse {
    pub fn new() -> Self {
        Self {}
    }
}

// Traits
impl ApiRequest for SocialRegisterInstanceRequest {
    type Response = SocialRegisterInstanceResponse;

    fn name() -> &'static str {
        "SocialRegisterInstanceRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "social/register_instance"
    }
}

impl ApiResponse for SocialRegisterInstanceResponse {
    fn name() -> &'static str {
        "SocialRegisterInstanceResponse"
    }
}
