use std::{net::SocketAddr, pin::Pin};

use async_dup::Arc;

use smol::{
    future::Future,
    lock::RwLock,
    net::{TcpListener},
};

use logging::info;
use http_common::{Request, Response, ResponseError};
use http_server_shared::{executor, serve_impl};

use crate::smol::io::{AsyncRead, AsyncWrite};
use crate::smol::stream::StreamExt;

pub struct RedirectServer {
    pub(crate) socket_addr: SocketAddr,
    endpoint: Option<
        Box<
            dyn 'static
            + Send
            + Sync
            + FnMut(
                (SocketAddr, Request),
            ) -> Pin<
                Box<
                    dyn 'static
                    + Send
                    + Sync
                    + Future<Output = Result<Response, ResponseError>>,
                >,
            >,
        >,
    >,
}

impl RedirectServer {
    pub fn new(socket_addr: SocketAddr) -> Self {
        Self {
            socket_addr,
            endpoint: None,
        }
    }

    pub fn start(self) {
        executor::spawn(async move {
            Self::listen(self).await;
        })
            .detach();
    }

    pub fn endpoint(
        &mut self,
        new_endpoint: Box<
            dyn Send
            + Sync
            + FnMut(
                (SocketAddr, Request),
            ) -> Pin<
                Box<dyn Send + Sync + Future<Output = Result<Response, ResponseError>>>,
            >,
        >,
    ) {
        self.endpoint = Some(new_endpoint);
    }

    /// Listens for incoming connections and serves them.
    async fn listen(server: Self) {
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
            let response_stream = response_stream
                .expect("unable to get the response stream");
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
    pub(crate) async fn serve<
        ResponseStream: Unpin + AsyncRead + AsyncWrite,
    >(
        server: Arc<RwLock<Self>>,
        incoming_address: SocketAddr,
        response_stream: ResponseStream,
    ) {
        let server_1 = server.clone();

        serve_impl(
            incoming_address,
            response_stream,
            |_key| {
                async move { true }
            },
            |(addr, request)| {
                let server_2 = server_1.clone();
                async move {
                    let mut server = server_2.write().await;
                    let endpoint = server.endpoint.as_mut().unwrap();

                    endpoint((addr, request)).await
                }
            },
        )
            .await;
    }
}