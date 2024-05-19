use naia_serde::SerdeInternal as Serde;

use bevy_http_shared::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct DisconnectSocialServerRequest {
    region_secret: String,
}

impl DisconnectSocialServerRequest {
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
pub struct DisconnectSocialServerResponse;

// Traits
impl ApiRequest for DisconnectSocialServerRequest {
    type Response = DisconnectSocialServerResponse;

    fn name() -> &'static str {
        "DisconnectSocialServerRequest"
    }

    fn method() -> Method {
        Method::Post
    }

    fn path() -> &'static str {
        "disconnect_social_server"
    }
}

impl ApiResponse for DisconnectSocialServerResponse {
    fn name() -> &'static str {
        "DisconnectSocialServerResponse"
    }
}
