use std::net::SocketAddr;

use executor::smol::{net::TcpListener, stream::StreamExt};

use config::{PUBLIC_IP_ADDR, PUBLIC_PROTOCOL};
use http_common::{Request, Response, ResponseError};
use http_server::log_util;
use http_server_shared::{serve_impl, MatchHostResult};
use logging::info;

pub struct RedirectServer;

impl RedirectServer {
    pub fn start(listen_addr: SocketAddr) {
        executor::spawn(async move {
            Self::listen(listen_addr).await;
        })
        .detach();
    }

    /// Listens for incoming connections and serves them.
    async fn listen(listen_addr: SocketAddr) {
        let listener = TcpListener::bind(listen_addr)
            .await
            .expect("unable to bind a TCP Listener to the supplied socket address");
        info!(
            "HTTP Listening at http://{}/",
            listener
                .local_addr()
                .expect("Listener does not have a local address"),
        );

        let mut incoming = listener.incoming();
        while let Some(response_stream) = incoming.next().await {
            let response_stream = response_stream.expect("unable to get the response stream");
            let incoming_address = response_stream
                .peer_addr()
                .expect("unable to get the peer address of the response stream");

            //info!("received request from {}", incoming_address);

            // Spawn a background task serving this connection.
            executor::spawn(async move {
                serve_impl(
                    incoming_address,
                    response_stream,
                    |_| async move { true },
                    |_, _| async move { MatchHostResult::Match },
                    |_, addr, req| RedirectServer::endpoint(addr, req),
                )
                .await;
            })
            .detach();
        }
        unreachable!()
    }

    async fn endpoint(
        socket_addr: SocketAddr,
        request: Request,
    ) -> Result<Response, ResponseError> {
        log_util::recv_req(
            "redirector",
            "client",
            format!(
                "[{}][{} {}]",
                socket_addr,
                request.method.as_str(),
                request.url
            )
            .as_str(),
        );

        let original_url = request.url.clone();
        let response = Response::redirect(
            &original_url,
            format!("{}://{}", PUBLIC_PROTOCOL, PUBLIC_IP_ADDR).as_str(),
        );

        log_util::send_res(
            "redirector",
            format!("redirect to {}", response.url).as_str(),
        );

        Ok(response)
    }
}
