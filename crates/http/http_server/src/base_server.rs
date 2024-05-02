use std::{collections::HashMap, net::SocketAddr};

use async_dup::Arc;

use executor::smol::{
    future::Future,
    io::{AsyncRead, AsyncWrite},
    lock::RwLock,
    net::TcpListener,
    stream::StreamExt,
};
use http_common::{Request, Response, ResponseError};
use http_server_shared::{executor, serve_impl, MatchHostResult};
use logging::info;

use crate::{endpoint::Endpoint, middleware::{RequestMiddlewareAction, RequestMiddleware, RequestMiddlewareFunc}};
use crate::middleware::{ResponseMiddleware, ResponseMiddlewareFunc};

// Server
pub struct Server {
    pub(crate) socket_addr: SocketAddr,
    endpoints: HashMap<String, Endpoint>,
    request_middlewares: Vec<RequestMiddleware>,
    response_middlewares: Vec<ResponseMiddleware>,
}

impl Server {
    pub fn new(socket_addr: SocketAddr) -> Self {
        Self {
            socket_addr,
            endpoints: HashMap::new(),
            request_middlewares: Vec::new(),
            response_middlewares: Vec::new(),
        }
    }

    pub fn start(self) {
        executor::spawn(async move {
            Self::listen(self).await;
        })
        .detach();
    }

    pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) {
        executor::spawn(future).detach();
    }

    pub fn request_middleware<
        ResponseType: 'static + Send + Sync + Future<Output = RequestMiddlewareAction>,
        Handler: 'static + Send + Sync + Fn(SocketAddr, Request) -> ResponseType
    >(
        &mut self,
        handler: Handler,
    ) {
        let func: RequestMiddlewareFunc = Box::new(move |addr, req| {
            Box::pin(handler(addr, req))
        });
        self.request_middlewares.push(RequestMiddleware::new(func));
    }

    pub fn response_middleware<
        ResponseType: 'static + Send + Sync + Future<Output = Result<Response, ResponseError>>,
        Handler: 'static + Send + Sync + Fn(Response) -> ResponseType
    >(
        &mut self,
        handler: Handler,
    ) {
        let func: ResponseMiddlewareFunc = Box::new(move |response| {
            Box::pin(handler(response))
        });
        self.response_middlewares.push(ResponseMiddleware::new(func));
    }

    // better know what you're doing here
    pub(crate) fn internal_insert_endpoint(
        &mut self,
        endpoint_path: String,
        new_endpoint: Endpoint,
    ) {
        self.endpoints.insert(endpoint_path, new_endpoint);
    }

    pub(crate) fn internal_endpoint_mut(&mut self, endpoint_path: &str) -> Option<&mut Endpoint> {
        self.endpoints.get_mut(endpoint_path)
    }

    /// Listens for incoming connections and serves them.
    async fn listen(server: Server) {
        let socket_addr = server.socket_addr;

        let listener = TcpListener::bind(socket_addr)
            .await
            .expect("unable to bind a TCP Listener to the supplied socket address");
        info!(
            "HTTP Listening at http://{}/",
            listener
                .local_addr()
                .expect("Listener does not have a local address"),
        );

        let server = Arc::new(RwLock::new(server));
        let mut incoming = listener.incoming();
        while let Some(response_stream) = incoming.next().await {
            let response_stream = response_stream.expect("unable to get the response stream");
            let incoming_address = response_stream
                .peer_addr()
                .expect("unable to get the peer address of the response stream");

            //info!("received request from {}", incoming_address);

            let self_clone = server.clone();

            // Spawn a background task serving this connection.
            executor::spawn(async move {
                Self::serve(self_clone, incoming_address, response_stream).await;
            })
            .detach();
        }
        unreachable!()
    }

    /// Reads a request from the client and sends it a response.
    pub(crate) async fn serve<ResponseStream: Unpin + AsyncRead + AsyncWrite>(
        server: Arc<RwLock<Server>>,
        incoming_address: SocketAddr,
        response_stream: ResponseStream,
    ) {
        let server_1 = server.clone();
        let server_2 = server.clone();

        serve_impl(
            incoming_address,
            response_stream,
            |endpoint_key| {
                let server_3 = server_1.clone();
                async move { server_3.read().await.endpoints.contains_key(&endpoint_key) }
            },
            |endpoint_key, host| {
                let server_3 = server_1.clone();
                async move {
                    let server = server_3.read().await;
                    let endpoint = server.endpoints.get(&endpoint_key).unwrap();
                    if let Some((required_host, redirect_url_opt)) = &endpoint.required_host {
                        if host.eq_ignore_ascii_case(required_host) {
                            MatchHostResult::Match
                        } else {
                            if let Some(redirect_url) = redirect_url_opt {
                                MatchHostResult::NoMatchRedirect(redirect_url.clone())
                            } else {
                                MatchHostResult::NoMatch
                            }
                        }
                    } else {
                        // if endpoint doesn't have a required host, then it's a match
                        MatchHostResult::Match
                    }
                }
            },
            |endpoint_key, address, request| {
                let server_4 = server_2.clone();
                async move {
                    let server = server_4.read().await;

                    let mut request = request.clone();

                    // handle global request middleware
                    for middleware in server.request_middlewares.iter() {
                        match (middleware.func)(address, request.clone()).await {
                            RequestMiddlewareAction::Continue(new_request) => {
                               request = new_request;
                            }
                            RequestMiddlewareAction::Stop(response) => {
                                return Ok(response);
                            }
                            RequestMiddlewareAction::Error(e) => {
                                return Err(e);
                            }
                        }
                    }

                    // handle request
                    let endpoint = server.endpoints.get(&endpoint_key).unwrap();
                    let response_result = endpoint.handle_request(address, request).await;

                    // handle global response middleware
                    match response_result {
                        Ok(mut response) => {
                            for middleware in server.response_middlewares.iter() {
                                match (middleware.func)(response.clone()).await {
                                    Ok(new_response) => {
                                        response = new_response;
                                    }
                                    Err(e) => {
                                        return Err(e);
                                    }
                                }
                            }
                            return Ok(response);
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            },
        )
        .await;
    }
}
