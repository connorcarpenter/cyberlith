use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct HeartbeatRequest {
    region_secret: String,
}

impl HeartbeatRequest {
    pub fn new(region_secret: &str) -> Self {
        Self {
            region_secret: region_secret.to_string(),
        }
    }

    pub fn region_secret(&self) -> &str {
        &self.region_secret
    }
}

// Response
#[derive(Serde, PartialEq, Clone)]
pub struct HeartbeatResponse;

// Traits
impl ApiRequest for HeartbeatRequest {
    type Response = HeartbeatResponse;

    fn name() -> &'static str { "HeartbeatRequest" }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "heartbeat"
    }
}

impl ApiResponse for HeartbeatResponse {
    fn name() -> &'static str { "HeartbeatResponse" }
}
