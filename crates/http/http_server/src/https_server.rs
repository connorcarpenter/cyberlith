use acme::Config;
use async_dup::Arc;
use http_server_shared::executor;
use logging::info;

use crate::{smol::lock::RwLock, smol::net::TcpListener, smol::stream::StreamExt, Server};

pub trait HttpsServer {
    fn https_start(self, config: Config);
}

impl HttpsServer for Server {
    fn https_start(self, config: Config) {
        executor::spawn(async move {
            https_listen(self, config).await;
        })
        .detach();
    }
}

/// Listens for incoming connections and serves them.
async fn https_listen(server: Server, config: Config) {
    let socket_addr = server.socket_addr;
    let acme_config = config.to_acme_config();

    let listener = TcpListener::bind(socket_addr)
        .await
        .expect("unable to bind a TCP Listener to the supplied socket address");
    info!(
        "HTTPS Listening at https://{}/",
        listener
            .local_addr()
            .expect("Listener does not have a local address"),
    );

    let server = Arc::new(RwLock::new(server));
    let mut incoming = acme_config.incoming(listener.incoming(), Vec::new());
    while let Some(response_stream) = incoming.next().await {
        let response_stream = response_stream.expect("unable to get the response stream");
        let (inner_stream, _server_connection) = response_stream.get_ref();
        let incoming_address = inner_stream
            .peer_addr()
            .expect("unable to get the peer address of the response stream");

        //info!("received request from {}", incoming_address);

        let self_clone = server.clone();

        // Spawn a background task serving this connection.
        executor::spawn(async move {
            Server::serve(self_clone, incoming_address, response_stream).await;
        })
        .detach();
    }
    unreachable!()
}
