
use crate::common::{Request, Response, ResponseError};

pub trait ApiRequest {
    type Response: ApiResponse;

    fn to_request(self) -> Request;
    fn from_request(request: Request) -> Result<Self, ()> where Self: Sized;
}

pub trait ApiResponse {
    fn to_response(self) -> Response;
    fn from_response(response: Response) -> Result<Self, ResponseError> where Self: Sized;
}
