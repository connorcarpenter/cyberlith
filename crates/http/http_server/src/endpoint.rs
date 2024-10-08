use std::{net::SocketAddr, pin::Pin};

use http_server_shared::executor::smol::future::Future;

use http_common::{Request, Response, ResponseError};

use crate::middleware::{RequestMiddlewareAction, ResponseMiddleware, ResponseMiddlewareFunc};
use crate::{
    middleware::{RequestMiddleware, RequestMiddlewareFunc},
    Server,
};

// Endpoint
pub(crate) type EndpointFunc = Box<
    dyn Send
        + Sync
        + Fn(
            SocketAddr,
            Request,
        ) -> Pin<Box<dyn Send + Sync + Future<Output = Result<Response, ResponseError>>>>,
>;

pub(crate) struct Endpoint {
    func: EndpointFunc,

    request_middlewares: Vec<RequestMiddleware>,
    response_middlewares: Vec<ResponseMiddleware>,

    // Option<(required_host, Option<redirect_url>)>
    pub(crate) required_host: Option<(String, Option<String>)>,
}

impl Endpoint {
    pub(crate) fn new(func: EndpointFunc, required_host: Option<(String, Option<String>)>) -> Self {
        Self {
            func,
            request_middlewares: Vec::new(),
            response_middlewares: Vec::new(),
            required_host,
        }
    }

    pub(crate) async fn handle_request(
        &self,
        address: SocketAddr,
        mut request: Request,
    ) -> Result<Response, ResponseError> {
        let mut set_cookies = Vec::new();

        // handle endpoint request middleware
        for middleware in self.request_middlewares.iter() {
            match (middleware.func)(address, request.clone()).await {
                RequestMiddlewareAction::Continue(new_request, set_cookie_opt) => {
                    request = new_request;
                    if let Some(set_cookie) = set_cookie_opt {
                        set_cookies.push(set_cookie);
                    }
                }
                RequestMiddlewareAction::Stop(response) => return Ok(response),
                RequestMiddlewareAction::Error(err) => return Err(err),
            }
        }

        match (self.func)(address, request).await {
            Ok(mut response) => {
                // handle endpoint response middleware
                for middleware in self.response_middlewares.iter() {
                    match (middleware.func)(response.clone()).await {
                        Ok(new_response) => {
                            response = new_response;
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                // handle set_cookies
                for cookie_vals in set_cookies {
                    for cookie_val in cookie_vals {
                        response.insert_header("Set-Cookie", &cookie_val);
                    }
                }
                return Ok(response);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

pub struct EndpointRef<'a> {
    server: &'a mut Server,
    key: String,
}

impl<'a> EndpointRef<'a> {
    pub fn new(server: &'a mut Server, key: String) -> Self {
        Self { server, key }
    }

    pub fn request_middleware<
        ResponseType: 'static + Send + Sync + Future<Output = RequestMiddlewareAction>,
        Handler: 'static + Send + Sync + Fn(SocketAddr, Request) -> ResponseType,
    >(
        self,
        handler: Handler,
    ) -> Self {
        let func: RequestMiddlewareFunc = Box::new(move |addr, req| Box::pin(handler(addr, req)));
        let endpoint = self.server.internal_endpoint_mut(&self.key).unwrap();
        endpoint
            .request_middlewares
            .push(RequestMiddleware::new(func));
        self
    }

    pub fn response_middleware<
        ResponseType: 'static + Send + Sync + Future<Output = Result<Response, ResponseError>>,
        Handler: 'static + Send + Sync + Fn(Response) -> ResponseType,
    >(
        self,
        handler: Handler,
    ) -> Self {
        let func: ResponseMiddlewareFunc = Box::new(move |response| Box::pin(handler(response)));
        let endpoint = self.server.internal_endpoint_mut(&self.key).unwrap();
        endpoint
            .response_middlewares
            .push(ResponseMiddleware::new(func));
        self
    }
}
