use std::{net::SocketAddr, pin::Pin};

use smol::future::Future;

use http_common::{Request, Response, ResponseError};

// Middleware
pub(crate) type MiddlewareFunc = Box<
    dyn Send
    + Sync
    + Fn(SocketAddr, Request) -> Pin<Box<dyn Send + Sync + Future<Output = Option<Result<Response, ResponseError>>>>>,
>;

pub struct Middleware {
    pub(crate) func: MiddlewareFunc,
}

impl Middleware {
    pub(crate) fn new(
        func: MiddlewareFunc,
    ) -> Self {
        Self {
            func,
        }
    }
}