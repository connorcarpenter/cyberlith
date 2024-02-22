use std::path::PathBuf;

use crate::common::{ApiRequest, ApiResponse, Request, Response};
use crate::ResponseError;

// Request
pub struct ReadRequest {
    pub path: PathBuf,
}

impl ReadRequest {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self {
            path: path.into(),
        }
    }
}

// Response
pub struct ReadResponse {
    pub bytes: Vec<u8>,
}

impl ReadResponse {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes
        }
    }
}

// Traits
impl ApiRequest for ReadRequest {
    type Response = ReadResponse;

    fn to_request(self) -> Request {
        Request::Read(self)
    }

    fn from_request(request: Request) -> Result<Self, ()> {
        let Request::Read(request) = request else {
            return Err(());
        };
        Ok(request)
    }
}

impl ApiResponse for ReadResponse {
    fn to_response(self) -> Response {
        Response::Read(self)
    }

    fn from_response(response: Response) -> Result<Self, ResponseError> {
        let Response::Read(response) = response else {
            return Err(ResponseError::InvalidResponse);
        };
        Ok(response)
    }
}