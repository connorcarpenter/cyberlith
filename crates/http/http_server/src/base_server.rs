use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, TcpStream},
    pin::Pin,
};

use async_dup::Arc;
use logging::info;
use smol::{future::Future, lock::RwLock, Async};

use http_common::{Request, Response, ResponseError};

use http_server_shared::{executor, serve_impl};

pub struct Server {
    socket_addr: SocketAddr,
    endpoints: HashMap<
        String,
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

impl Server {
    pub fn new(socket_addr: SocketAddr) -> Self {
        Server {
            socket_addr,
            endpoints: HashMap::new(),
        }
    }

    pub fn start(self) {
        let self_ref = Arc::new(RwLock::new(self));
        executor::spawn(async move {
            listen(self_ref).await;
        })
        .detach();
    }

    pub fn spawn<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) {
        executor::spawn(future).detach();
    }

    // better know what you're doing here
    pub fn internal_insert_endpoint(
        &mut self,
        endpoint_path: String,
        new_endpoint: Box<dyn Send + Sync + FnMut((SocketAddr, Request)) -> Pin<Box<dyn Send + Sync + Future<Output=Result<Response, ResponseError>>>>>
    ) {
        self.endpoints.insert(endpoint_path, new_endpoint);
    }
}

/// Listens for incoming connections and serves them.
async fn listen(server: Arc<RwLock<Server>>) {
    let socket_addr = server.read().await.socket_addr;
    let listener = Async::<TcpListener>::bind(socket_addr)
        .expect("unable to bind a TCP Listener to the supplied socket address");
    info!(
        "HTTP Listening at http://{}/",
        listener
            .get_ref()
            .local_addr()
            .expect("Listener does not have a local address"),
    );

    loop {
        // Accept the next connection.
        let (response_stream, incoming_address) = listener
            .accept()
            .await
            .expect("was not able to accept the incoming stream from the listener");

        //info!("received request from {}", incoming_address);

        let self_clone = server.clone();

        // Spawn a background task serving this connection.
        executor::spawn(async move {
            serve(self_clone, incoming_address, Arc::new(response_stream)).await;
        })
        .detach();
    }
}

/// Reads a request from the client and sends it a response.
async fn serve(
    server: Arc<RwLock<Server>>,
    incoming_address: SocketAddr,
    response_stream: Arc<Async<TcpStream>>,
) {
    let endpoint_key_ref: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

    let server_1 = server.clone();
    let server_2 = server.clone();

    let endpoint_key_ref_1 = endpoint_key_ref.clone();
    let endpoint_key_ref_2 = endpoint_key_ref.clone();

    serve_impl(
        incoming_address,
        response_stream,
        |key| {
            let server_3 = server_1.clone();
            let endpoint_key_ref_3 = endpoint_key_ref_1.clone();
            async move {
                let server = server_3.read().await;
                if server.endpoints.contains_key(&key) {
                    let mut endpoint_key = endpoint_key_ref_3.write().await;
                    *endpoint_key = Some(key);
                    true
                } else {
                    false
                }
            }
        },
        |(addr, request)| {
            let server_4 = server_2.clone();
            let endpoint_key_ref_4 = endpoint_key_ref_2.clone();
            async move {
                let endpoint_key = endpoint_key_ref_4.read().await.as_ref().unwrap().clone();
                let mut server = server_4.write().await;
                let endpoint = server.endpoints.get_mut(&endpoint_key).unwrap();

                endpoint((addr, request)).await
            }
        },
    )
    .await;
}