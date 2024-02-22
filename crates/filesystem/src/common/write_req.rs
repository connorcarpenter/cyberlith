use std::path::PathBuf;

use crate::common::{ApiRequest, ApiResponse, Request, Response};
use crate::ResponseError;

// Request
pub struct WriteRequest {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

impl WriteRequest {
    pub fn new<T: Into<PathBuf>>(path: T, bytes: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            bytes,
        }
    }
}

// Response
pub struct WriteResponse;

impl WriteResponse {
    pub fn new() -> Self {
        Self
    }
}

// Traits
impl ApiRequest for WriteRequest {
    type Response = WriteResponse;

    fn to_request(self) -> Request {
        Request::Write(self)
    }

    fn from_request(request: Request) -> Result<Self, ()> {
        let Request::Write(request) = request else {
            return Err(());
        };
        Ok(request)
    }
}

impl ApiResponse for WriteResponse {
    fn to_response(self) -> Response {
        Response::Write(self)
    }

    fn from_response(response: Response) -> Result<Self, ResponseError> {
        let Response::Write(response) = response else {
            return Err(ResponseError::InvalidResponse);
        };
        Ok(response)
    }
}