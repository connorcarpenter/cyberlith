use std::{
    collections::{BTreeMap, HashMap},
    net::{SocketAddr, TcpListener, TcpStream},
    pin::Pin,
};

use async_dup::Arc;
use log::{info, warn};
use smol::{
    Async,
    future::Future,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    lock::RwLock,
    stream::StreamExt,
};

use http_common::{ApiRequest, ApiResponse, Method, Request, Response};

use http_server_shared::{executor, ReadState};

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
async fn serve(server: Arc<RwLock<Server>>, mut response_stream: Arc<Async<TcpStream>>) {
    let mut method: Option<Method> = None;
    let mut uri: Option<String> = None;
    let mut endpoint_key: Option<String> = None;
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
                    //info!("finished reading body");
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

            //info!("read: {}", str);

            match read_state {
                ReadState::MatchingUrl => {
                    let parts = str.split(" ").collect::<Vec<&str>>();
                    let key = format!("{} {}", parts[0], parts[1]);
                    let server = server.read().await;
                    if server.endpoints.contains_key(&key) {
                        read_state = ReadState::ReadingHeaders;
                        endpoint_key = Some(key);
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
                        //info!("finished reading headers.");

                        read_state = ReadState::ReadingBody;

                        let Some(content_length) = content_length else {
                            warn!("request was missing Content-Length header");
                            read_state = ReadState::Error;
                            break;
                        };
                        if content_length == 0 {
                            read_state = ReadState::Finished;
                            //info!("no body to read. finished.");
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

    // cast to request //
    let Some(method) = method else {
        warn!("unable to parse method");
        return send_404(response_stream).await;
    };
    let Some(uri) = uri else {
        warn!("unable to parse uri");
        return send_404(response_stream).await;
    };
    let mut request = Request::new(method, &uri, body);
    request.headers = header_map;

    let endpoint_key = endpoint_key.unwrap();
    let server = server.read().await;
    let endpoint = server.endpoints.get(&endpoint_key).unwrap();

    let response_result = endpoint(request).await;

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
