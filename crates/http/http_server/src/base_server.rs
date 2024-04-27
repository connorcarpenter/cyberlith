use std::{collections::HashMap, net::SocketAddr, pin::Pin};

use async_dup::Arc;
use smol::{
    future::Future,
    io::{AsyncRead, AsyncWrite},
    lock::RwLock,
    net::TcpListener,
    stream::StreamExt,
};

use http_common::{Request, Response, ResponseError};
use http_server_shared::{executor, serve_impl, MatchHostResult};
use logging::info;

// Endpoint
pub(crate) type EndpointFunc = Box<
    dyn Send
        + Sync
        + Fn(
        // TODO: get rid of these parenthesis...
            (SocketAddr, Request),
        ) -> Pin<Box<dyn Send + Sync + Future<Output = Result<Response, ResponseError>>>>,
>;

pub(crate) struct Endpoint {
    func: EndpointFunc,
    // Option<(required_host, Option<redirect_url>)>
    required_host: Option<(String, Option<String>)>,
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

// Middleware
pub(crate) type MiddlewareFunc = Box<
    dyn Send
    + Sync
    + Fn(
        SocketAddr, Request,
    ) -> Pin<Box<dyn Send + Sync + Future<Output = Option<Result<Response, ResponseError>>>>>,
>;

pub struct Middleware {
    func: MiddlewareFunc,
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

// Server
pub struct Server {
    pub(crate) socket_addr: SocketAddr,
    endpoints: HashMap<String, Endpoint>,
    middlewares: Vec<Middleware>,
}

impl Server {
    pub fn new(socket_addr: SocketAddr) -> Self {
        Self {
            socket_addr,
            endpoints: HashMap::new(),
            middlewares: Vec::new(),
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

    pub fn middleware<
        ResponseType: 'static + Send + Sync + Future<Output = Option<Result<Response, ResponseError>>>,
        Handler: 'static + Send + Sync + Fn(SocketAddr, Request) -> ResponseType
    >(
        &mut self,
        handler: Handler,
    ) {
        let func: MiddlewareFunc = Box::new(move |addr, req| {
            Box::pin(handler(addr, req))
        });
        self.middlewares.push(Middleware::new(func));
    }

    // better know what you're doing here
    pub(crate) fn internal_insert_endpoint(
        &mut self,
        endpoint_path: String,
        new_endpoint: Endpoint,
    ) {
        self.endpoints.insert(endpoint_path, new_endpoint);
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
            |endpoint_key, addr, request| {
                let server_4 = server_2.clone();
                async move {
                    let server = server_4.read().await;

                    for middleware in server.middlewares.iter() {
                        if let Some(response_result) = (middleware.func)(addr, request.clone()).await {
                            return response_result;
                        }
                    }

                    let endpoint = server.endpoints.get(&endpoint_key).unwrap();

                    (endpoint.func)((addr, request)).await
                }
            },
        )
        .await;
    }
}
