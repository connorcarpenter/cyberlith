use std::{net::SocketAddr, pin::Pin};

use smol::future::Future;

use http_common::{Request, Response, ResponseError};

// Endpoint
pub(crate) type EndpointFunc = Box<
    dyn Send
    + Sync
    + Fn(SocketAddr, Request) -> Pin<Box<dyn Send + Sync + Future<Output = Result<Response, ResponseError>>>>,
>;

pub(crate) struct Endpoint {
    pub(crate) func: EndpointFunc,
    // Option<(required_host, Option<redirect_url>)>
    pub(crate) required_host: Option<(String, Option<String>)>,
}

impl Endpoint {
    pub(crate) fn new(
        func: EndpointFunc,
        required_host: Option<(String, Option<String>)>,
    ) -> Self {
        Self {
            func,
            required_host,
        }
    }
}