use std::{net::SocketAddr, pin::Pin};

use http_server_shared::executor::smol::future::Future;

use http_common::{Request, Response, ResponseError};

pub enum RequestMiddlewareAction {
    Continue(Request),
    Stop(Response),
    Error(ResponseError),
}

// Request Middleware
pub(crate) type RequestMiddlewareFunc = Box<
    dyn Send
    + Sync
    + Fn(SocketAddr, Request) -> Pin<Box<dyn Send + Sync + Future<Output = RequestMiddlewareAction>>>,
>;

pub struct RequestMiddleware {
    pub(crate) func: RequestMiddlewareFunc,
}

impl RequestMiddleware {
    pub(crate) fn new(
        func: RequestMiddlewareFunc,
    ) -> Self {
        Self {
            func,
        }
    }
}

// Response Middleware
pub(crate) type ResponseMiddlewareFunc = Box<
    dyn Send
    + Sync
    + Fn(Response) -> Pin<Box<dyn Send + Sync + Future<Output = Result<Response, ResponseError>>>>,
>;

pub struct ResponseMiddleware {
    pub(crate) func: ResponseMiddlewareFunc,
}

impl ResponseMiddleware {
    pub(crate) fn new(
        func: ResponseMiddlewareFunc,
    ) -> Self {
        Self {
            func,
        }
    }
}