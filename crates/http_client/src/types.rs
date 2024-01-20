use http::{Request as HttpRequest, Response as HttpResponse, response::Builder as HttpResponseBuilder};

pub type Request = HttpRequest<String>;
pub type Response = HttpResponse<String>;
pub type ResponseBuilder = HttpResponseBuilder;

pub use http::header;