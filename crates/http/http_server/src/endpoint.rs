use std::{net::SocketAddr, pin::Pin};

use smol::future::Future;

use http_common::{Request, Response, ResponseError};

use crate::{Server, middleware::{Middleware, MiddlewareFunc}};

// Endpoint
pub(crate) type EndpointFunc = Box<
    dyn Send
    + Sync
    + Fn(SocketAddr, Request) -> Pin<Box<dyn Send + Sync + Future<Output = Result<Response, ResponseError>>>>,
>;

pub(crate) struct Endpoint {
    func: EndpointFunc,

    middlewares: Vec<Middleware>,

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
            middlewares: Vec::new(),
            required_host,
        }
    }

    pub(crate) async fn handle_request(
        &self,
        address: SocketAddr,
        request: Request,
    ) -> Result<Response, ResponseError> {

        // handle endpoint middleware
        for middleware in self.middlewares.iter() {
            if let Some(response_result) = (middleware.func)(address, request.clone()).await {
                return response_result;
            }
        }

        (self.func)(address, request).await
    }
}

pub struct EndpointRef<'a> {
    server: &'a mut Server,
    key: String,
}

impl<'a> EndpointRef<'a> {
    pub fn new(server: &'a mut Server, key: String) -> Self {
        Self {
            server,
            key,
        }
    }

    pub fn middleware<
        ResponseType: 'static + Send + Sync + Future<Output = Option<Result<Response, ResponseError>>>,
        Handler: 'static + Send + Sync + Fn(SocketAddr, Request) -> ResponseType
    >(
        self,
        handler: Handler,
    ) -> Self {
        let func: MiddlewareFunc = Box::new(move |addr, req| {
            Box::pin(handler(addr, req))
        });
        let endpoint = self.server.internal_endpoint_mut(&self.key).unwrap();
        endpoint.middlewares.push(Middleware::new(func));
        self
    }
}