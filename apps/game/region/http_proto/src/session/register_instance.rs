
use naia_serde::{SerdeInternal as Serde};

use http_common::{ApiRequest, ApiResponse, Method};

// Request
#[derive(Serde, PartialEq, Clone)]
pub struct SessionRegisterInstanceRequest;

impl SessionRegisterInstanceRequest {
    pub fn new() -> Self {
        Self {

        }
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
