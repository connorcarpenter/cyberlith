use naia_serde::SerdeInternal as Serde;

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SessionRegisterInstanceRequest {
    global_secret: String,
    instance_secret: String,
    http_addr: String,
    http_port: u16,
    public_webrtc_url: String,
}

impl SessionRegisterInstanceRequest {
    pub fn new(
        global_secret: &str,
        instance_secret: &str,
        http_addr: &str,
        http_port: u16,
        public_webrtc_url: &str,
    ) -> Self {
        Self {
            global_secret: global_secret.to_string(),
            instance_secret: instance_secret.to_string(),
            http_addr: http_addr.to_string(),
            http_port,
            public_webrtc_url: public_webrtc_url.to_string(),
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

    pub fn public_webrtc_url(&self) -> &str {
        &self.public_webrtc_url
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct SessionRegisterInstanceResponse;

impl SessionRegisterInstanceResponse {
    pub fn new() -> Self {
        Self {}
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
