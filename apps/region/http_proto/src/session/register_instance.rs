
use naia_serde::{SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SessionRegisterInstanceRequest {
    session_secret: String,
    http_addr: String,
    http_port: u16,
    signal_addr: String,
    signal_port: u16,
}

impl SessionRegisterInstanceRequest {
    pub fn new(
        session_secret: &str,
        http_addr: &str,
        http_port: u16,
        signal_addr: &str,
        signal_port: u16,
    ) -> Self {
        Self {
            session_secret: session_secret.to_string(),
            http_addr: http_addr.to_string(),
            http_port,
            signal_addr: signal_addr.to_string(),
            signal_port,
        }
    }

    pub fn session_secret(&self) -> &str {
        &self.session_secret
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
        self.signal_port.clone()
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
