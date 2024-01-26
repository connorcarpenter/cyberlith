use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener, TcpStream},
    any::TypeId
};

use async_dup::Arc;
use log::info;
use smol::{channel::{Receiver, Sender}, lock::RwLock, Async, channel};

use bevy_http_shared::Protocol;
use http_common::{Request, Response, ResponseError};
use http_server_shared::{executor, serve_impl};

struct KeyMaker {
    current_index: u64,
}

impl KeyMaker {
    pub fn new() -> Self {
        Self {
            current_index: 0,
        }
    }

    pub fn next_key_id(&mut self) -> u64 {
        let next_index = self.current_index;
        self.current_index = self.current_index.wrapping_add(1);
        next_index
    }
}

pub struct ServerState {
    protocol: Protocol,
    request_senders: HashMap<TypeId, Sender<(u64, SocketAddr, Request)>>,
    main_response_receiver: Option<Receiver<(u64, Result<Response, ResponseError>)>>,
    response_senders: HashMap<u64, Sender<Result<Response, ResponseError>>>,
    key_maker: KeyMaker,
}

impl ServerState {
    pub fn new(protocol: Protocol) -> (Self, HashMap<TypeId, Receiver<(u64, SocketAddr, Request)>>, Sender<(u64, Result<Response, ResponseError>)>) {

        // Requests
        let mut request_senders = HashMap::new();
        let mut request_receivers = HashMap::new();
        let types = protocol.get_all_types();
        for type_id in types {

            let (request_sender, request_receiver) = channel::unbounded();

            request_senders.insert(type_id, request_sender);
            request_receivers.insert(type_id, request_receiver);
        }

        // Responses
        let (response_sender, response_receiver) = channel::unbounded();

        let me = Self {
            protocol,
            request_senders,
            main_response_receiver: Some(response_receiver),
            response_senders: HashMap::new(),
            key_maker: KeyMaker::new(),
        };

        (me, request_receivers, response_sender)
    }

    pub fn listen(self, addr: SocketAddr) {
        let ServerState {
            protocol,
            request_senders,
            main_response_receiver,
            response_senders,
            key_maker,
        } = self;

        let main_response_receiver = main_response_receiver.unwrap();
        let response_senders = Arc::new(RwLock::new(response_senders));
        let response_senders_clone = response_senders.clone();
        let key_maker = Arc::new(RwLock::new(key_maker));

        // needs protocol, request senders, and response senders
        executor::spawn(async move {
            listen(addr, protocol, request_senders, response_senders_clone, key_maker).await;
        })
            .detach();

        executor::spawn(async move {
            process_responses(main_response_receiver, response_senders).await;
        })
            .detach();
    }
}

/// Listens for incoming connections and serves them.
// needs protocol, request senders, and response senders
async fn listen(
    listen_address: SocketAddr,
    protocol: Protocol,
    request_senders: HashMap<TypeId, Sender<(u64, SocketAddr, Request)>>,
    response_senders_map: Arc<RwLock<HashMap<u64, Sender<Result<Response, ResponseError>>>>>,
    key_maker: Arc<RwLock<KeyMaker>>,
) {
    let listener = Async::<TcpListener>::bind(listen_address)
        .expect("unable to bind a TCP Listener to the supplied socket address");
    info!(
        "HTTP Listening at http://{}/",
        listener
            .get_ref()
            .local_addr()
            .expect("Listener does not have a local address"),
    );

    let protocol = Arc::new(protocol);
    let request_senders = Arc::new(request_senders);

    loop {
        // Accept the next connection.
        let (response_stream, incoming_address) = listener
            .accept()
            .await
            .expect("was not able to accept the incoming stream from the listener");

        //info!("received request from {} .. making new thread to serve this connection", incoming_address);

        let protocol_clone = protocol.clone();
        let request_senders_clone = request_senders.clone();
        let response_senders_map_clone = response_senders_map.clone();
        let key_maker_clone = key_maker.clone();

        // Spawn a background task serving this connection.
        executor::spawn(async move {
            serve(
                incoming_address,
                Arc::new(response_stream),
                protocol_clone,
                request_senders_clone,
                response_senders_map_clone,
                key_maker_clone,
            ).await;
        })
            .detach();
    }
}

// needs response_senders
async fn process_responses(
    response_receiver: Receiver<(u64, Result<Response, ResponseError>)>,
    response_senders_map: Arc<RwLock<HashMap<u64, Sender<Result<Response, ResponseError>>>>>
) {

    loop {
        let (response_id, response) = response_receiver.recv().await.expect("unable to receive response");
        let mut response_senders = response_senders_map.write().await;

        let Some(response_sender) = response_senders.remove(&response_id) else {
            panic!("received response for unknown response id: {}", response_id);
        };
        response_sender.try_send(response).expect("unable to send response");
    }
}

/// Reads a request from the client and sends it a response.
// needs protocol, request senders, and response senders
async fn serve(
    incoming_addr: SocketAddr,
    response_stream: Arc<Async<TcpStream>>,
    protocol: Arc<Protocol>,
    request_senders: Arc<HashMap<TypeId, Sender<(u64, SocketAddr, Request)>>>,
    response_senders: Arc<RwLock<HashMap<u64, Sender<Result<Response, ResponseError>>>>>,
    key_maker: Arc<RwLock<KeyMaker>>,
) {
    let endpoint_key_ref: Arc<RwLock<Option<TypeId>>> = Arc::new(RwLock::new(None));

    let protocol_1 = protocol.clone();
    let keymaker_1 = key_maker.clone();
    let request_senders_1 = request_senders.clone();
    let response_senders_1 = response_senders.clone();

    let endpoint_key_ref_1 = endpoint_key_ref.clone();
    let endpoint_key_ref_2 = endpoint_key_ref.clone();

    serve_impl(
        incoming_addr,
        response_stream,
        |key| {
            let protocol_2 = protocol_1.clone();
            let endpoint_key_ref_3 = endpoint_key_ref_1.clone();
            async move {
                //info!("attempting to match url. endpoint key is: {}", key);

                if protocol_2.has_endpoint_key(&key) {
                    let request_id_temp = protocol_2.get_request_id(&key).unwrap();
                    let mut endpoint_key = endpoint_key_ref_3.write().await;
                    *endpoint_key = Some(request_id_temp);
                    true
                } else {
                    false
                }
            }
        },
        |(req_addr, request)| {

            let endpoint_key_ref_4 = endpoint_key_ref_2.clone();
            let keymaker_2 = keymaker_1.clone();
            let request_senders_2 = request_senders_1.clone();
            let response_senders_2 = response_senders_1.clone();
            async move {
                let endpoint_key = endpoint_key_ref_4.read().await.as_ref().unwrap().clone();

                let response_receiver = {
                    let mut key_maker = keymaker_2.write().await;
                    let response_key_id = key_maker.next_key_id();

                    let Some(request_sender) = request_senders_2.get(&endpoint_key) else {
                        panic!("did not register type!");
                    };
                    request_sender.try_send((response_key_id, req_addr, request)).expect("unable to send request");

                    let (response_sender, response_receiver) = channel::bounded(1);
                    let mut response_senders = response_senders_2.write().await;
                    response_senders.insert(response_key_id, response_sender);

                    response_receiver
                };

                let response_result = match response_receiver.recv().await {
                    Ok(response_result) => {
                        response_result
                    }
                    Err(_err) => {
                        Err(ResponseError::ChannelRecvError)
                    }
                };

                response_result
            }
        }
    ).await;
}