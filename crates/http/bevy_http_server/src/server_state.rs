use std::{
    collections::{BTreeMap, HashMap},
    net::{SocketAddr, TcpListener, TcpStream},
    any::TypeId
};

use async_dup::Arc;
use log::{info, warn};
use smol::{channel::{Receiver, Sender}, io::{AsyncReadExt, AsyncWriteExt, BufReader}, lock::RwLock, stream::StreamExt, Async, channel};

use bevy_http_shared::Protocol;

use http_common::{Method, Request, Response};

use crate::executor;

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
    main_response_receiver: Option<Receiver<(u64, Response)>>,
    response_senders: HashMap<u64, Sender<Response>>,
    key_maker: KeyMaker,
}

impl ServerState {
    pub fn new(protocol: Protocol) -> (Self, HashMap<TypeId, Receiver<(u64, SocketAddr, Request)>>, Sender<(u64, Response)>) {

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
    response_senders_map: Arc<RwLock<HashMap<u64, Sender<Response>>>>,
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

        info!("received request from {} .. making new thread to serve this connection", incoming_address);

        let protocol_clone = protocol.clone();
        let request_senders_clone = request_senders.clone();
        let response_senders_map_clone = response_senders_map.clone();
        let key_maker_clone = key_maker.clone();

        // Spawn a background task serving this connection.
        executor::spawn(async move {
            serve(
                Arc::new(response_stream),
                incoming_address,
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
    response_receiver: Receiver<(u64, Response)>,
    response_senders_map: Arc<RwLock<HashMap<u64, Sender<Response>>>>
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

#[derive(PartialEq, Eq)]
enum ReadState {
    MatchingUrl,
    ReadingHeaders,
    ReadingBody,
    Finished,
    Error,
}

/// Reads a request from the client and sends it a response.
// needs protocol, request senders, and response senders
async fn serve(
    mut response_stream: Arc<Async<TcpStream>>,
    request_addr: SocketAddr,
    protocol: Arc<Protocol>,
    request_senders: Arc<HashMap<TypeId, Sender<(u64, SocketAddr, Request)>>>,
    response_senders: Arc<RwLock<HashMap<u64, Sender<Response>>>>,
    key_maker: Arc<RwLock<KeyMaker>>,
) {
    let mut method: Option<Method> = None;
    let mut uri: Option<String> = None;
    let mut request_id: Option<TypeId> = None;
    let mut content_length: Option<usize> = None;
    let mut body: Vec<u8> = Vec::new();
    let mut header_map = BTreeMap::<String, String>::new();

    let mut read_state = ReadState::MatchingUrl;

    let buf_reader = BufReader::new(response_stream.clone());

    let mut bytes = buf_reader.bytes();

    let mut line: Vec<u8> = Vec::new();

    loop {
        let Some(byte) = bytes.next().await else {
            info!("no more bytes!");
            break;
        };

        let byte = byte.expect("unable to read a byte from incoming stream");

        if read_state == ReadState::ReadingBody {
            //info!("read byte from body");

            if let Some(content_length) = content_length {
                body.push(byte);

                if body.len() >= content_length {
                    read_state = ReadState::Finished;
                    info!("finished reading body");
                    break;
                }

                continue;
            } else {
                warn!("request was missing Content-Length header");
                read_state = ReadState::Error;
                break;
            }
        }

        if byte == b'\r' {
            continue;
        } else if byte == b'\n' {
            let str =
                String::from_utf8(line.clone()).expect("unable to parse string from UTF-8 bytes");
            line.clear();

            info!("read: {}", str);

            match read_state {
                ReadState::MatchingUrl => {
                    let parts = str.split(" ").collect::<Vec<&str>>();
                    let key = format!("{} {}", parts[0], parts[1]);

                    info!("attempting to match url. endpoint key is: {}", key);

                    info!("got server, checking.");

                    if protocol.has_endpoint_key(&key) {
                        read_state = ReadState::ReadingHeaders;
                        let request_id_temp = protocol.get_request_id(&key).unwrap();
                        request_id = Some(request_id_temp);
                        method = Some(Method::from_str(parts[0]).unwrap());
                        uri = Some(parts[1].to_string());
                    } else {
                        warn!("no endpoint found for {}", key);
                        read_state = ReadState::Error;
                        break;
                    }
                }
                ReadState::ReadingHeaders => {
                    if str.is_empty() {
                        info!("finished reading headers.");

                        read_state = ReadState::ReadingBody;

                        let Some(content_length) = content_length else {
                            warn!("request was missing Content-Length header");
                            read_state = ReadState::Error;
                            break;
                        };
                        if content_length == 0 {
                            read_state = ReadState::Finished;
                            info!("no body to read. finished.");
                            break;
                        } else {
                            continue;
                        }
                    } else {
                        let parts = str.split(": ").collect::<Vec<&str>>();
                        header_map.insert(parts[0].to_string(), parts[1].to_string());
                        if parts[0].to_lowercase() == "content-length" {
                            content_length = Some(parts[1].parse().unwrap());
                        }
                    }
                }
                _ => {
                    warn!("shouldn't be in this state");
                    return send_404(response_stream).await;
                }
            }
        } else {
            line.push(byte);
        }
    }

    if read_state != ReadState::Finished {
        return send_404(response_stream).await;
    }

    // done reading //

    info!("done reading request");

    // cast to request //
    let Some(method) = method else {
        warn!("unable to parse method");
        return send_404(response_stream).await;
    };
    let Some(uri) = uri else {
        warn!("unable to parse uri");
        return send_404(response_stream).await;
    };

    info!("done casting request");

    let mut request = Request::new(method, &uri, body);
    request.headers = header_map;

    let request_id = request_id.unwrap();

    info!("sending request");

    let response_receiver = {
        let mut key_maker = key_maker.write().await;
        let response_key_id = key_maker.next_key_id();

        let Some(request_sender) = request_senders.get(&request_id) else {
            panic!("did not register type!");
        };
        request_sender.try_send((response_key_id, request_addr, request)).expect("unable to send request");

        let (response_sender, response_receiver) = channel::bounded(1);
        let mut response_senders = response_senders.write().await;
        response_senders.insert(response_key_id, response_sender);

        response_receiver
    };

    info!("waiting for response");

    let response_result = response_receiver.recv().await;

    info!("response received");

    match response_result {
        Ok(mut response) => {
            response
                .headers
                .insert("Access-Control-Allow-Origin".to_string(), "*".to_string());

            let mut response_bytes = response_header_to_vec(&response);
            response_bytes.extend_from_slice(&response.body);
            response_stream
                .write_all(&response_bytes)
                .await
                .expect("found an error while writing to a stream");

            response_stream
                .flush()
                .await
                .expect("unable to flush the stream");
            response_stream
                .close()
                .await
                .expect("unable to close the stream");
        }
        Err(_e) => {
            return send_404(response_stream).await;
        }
    }
}

const RESPONSE_BAD: &[u8] = br#"
HTTP/1.1 404 NOT FOUND
Content-Type: text/html
Content-Length: 0
Access-Control-Allow-Origin: *
"#;

async fn send_404(mut response_stream: Arc<Async<TcpStream>>) {
    response_stream.write_all(RESPONSE_BAD).await.unwrap();
    response_stream
        .flush()
        .await
        .expect("unable to flush the stream");
    response_stream
        .close()
        .await
        .expect("unable to close the stream");
}

fn response_header_to_vec(r: &Response) -> Vec<u8> {
    let v = Vec::with_capacity(120);
    let mut c = std::io::Cursor::new(v);
    write_response_header(r, &mut c).expect("unable to write response header to stream");
    c.into_inner()
}

fn write_response_header(r: &Response, mut io: impl std::io::Write) -> std::io::Result<usize> {
    let mut len = 0;

    let status = r.status;
    let code = status.to_string();
    let reason = "Unknown";
    let headers = &r.headers;

    write_line(&mut io, &mut len, b"HTTP/1.1 ")?;
    write_line(&mut io, &mut len, code.as_bytes())?;
    write_line(&mut io, &mut len, b" ")?;
    write_line(&mut io, &mut len, reason.as_bytes())?;
    write_line(&mut io, &mut len, b"\r\n")?;

    for (hn, hv) in headers {
        write_line(&mut io, &mut len, hn.as_str().as_bytes())?;
        write_line(&mut io, &mut len, b": ")?;
        write_line(&mut io, &mut len, hv.as_bytes())?;
        write_line(&mut io, &mut len, b"\r\n")?;
    }

    write_line(&mut io, &mut len, b"\r\n")?;

    Ok(len)
}

fn write_line(io: &mut dyn std::io::Write, len: &mut usize, buf: &[u8]) -> std::io::Result<()> {
    io.write_all(buf)?;
    *len += buf.len();
    Ok(())
}
