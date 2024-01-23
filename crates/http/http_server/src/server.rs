use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, TcpStream},
    pin::Pin,
};

use async_dup::Arc;
use log::info;
use smol::{
    Async,
    future::Future,
    lock::RwLock,
};

use http_common::{ApiRequest, ApiResponse, Request, Response};

use http_server_shared::{executor, serve_impl};

pub struct Server {
    socket_addr: SocketAddr,
    endpoints: HashMap<
        String,
        Box<
            dyn 'static
                + Send
                + Sync
                + Fn(
                    Request,
                )
                    -> Pin<Box<dyn 'static + Send + Sync + Future<Output = Result<Response, ()>>>>,
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

    pub fn endpoint<
        TypeRequest: 'static + ApiRequest,
        TypeResponse: 'static + Send + Sync + Future<Output = Result<TypeRequest::Response, ()>>,
    >(
        &mut self,
        handler: fn(TypeRequest) -> TypeResponse,
    ) {
        let method = TypeRequest::method();
        let path = TypeRequest::path();

        let endpoint_path = format!("{} /{}", method.as_str(), path);

        info!("endpoint: {}", endpoint_path);
        let new_endpoint = endpoint_2::<TypeRequest, TypeResponse>(handler);
        self.endpoints.insert(endpoint_path, new_endpoint);
    }
}

fn endpoint_2<
    TypeRequest: 'static + ApiRequest,
    TypeResponse: 'static + Send + Sync + Future<Output = Result<TypeRequest::Response, ()>>,
>(
    handler: fn(TypeRequest) -> TypeResponse,
) -> Box<
    dyn 'static
        + Send
        + Sync
        + Fn(Request) -> Pin<Box<dyn 'static + Send + Sync + Future<Output = Result<Response, ()>>>>,
> {
    Box::new(move |pure_request: Request| {
        let Ok(typed_request) = TypeRequest::from_request(pure_request) else {
                panic!("unable to convert request to typed request, handle this better in future!");
            };

        let typed_future = handler(typed_request);

        // convert typed future to pure future
        let pure_future = async move {
            let typed_response = typed_future.await;
            match typed_response {
                Ok(typed_response) => {
                    let pure_response = typed_response.to_response();
                    Ok(pure_response)
                }
                Err(_) => Err(()),
            }
        };

        Box::pin(pure_future)
    })
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
        let (response_stream, _incoming_address) = listener
            .accept()
            .await
            .expect("was not able to accept the incoming stream from the listener");

        //info!("received request from {}", incoming_address);

        let self_clone = server.clone();

        // Spawn a background task serving this connection.
        executor::spawn(async move {
            serve(self_clone, Arc::new(response_stream)).await;
        })
        .detach();
    }
}

/// Reads a request from the client and sends it a response.
async fn serve(
    server: Arc<RwLock<Server>>,
    response_stream: Arc<Async<TcpStream>>
) {
    let endpoint_key_ref: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

    let server_1 = server.clone();
    let server_2 = server.clone();

    let endpoint_key_ref_1 = endpoint_key_ref.clone();
    let endpoint_key_ref_2 = endpoint_key_ref.clone();

    serve_impl(
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
        |request| {
            let server_4 = server_2.clone();
            let endpoint_key_ref_4 = endpoint_key_ref_2.clone();
            async move {
                let endpoint_key = endpoint_key_ref_4.read().await.as_ref().unwrap().clone();
                let server = server_4.read().await;
                let endpoint = server.endpoints.get(&endpoint_key).unwrap();

                endpoint(request).await
            }
        }
    ).await;
}