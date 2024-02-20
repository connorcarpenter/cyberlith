use std::{
    collections::BTreeMap,
    net::{SocketAddr, TcpStream},
};

use async_dup::Arc;
use log::{info, warn};
use smol::{
    future::Future,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    stream::StreamExt,
    Async,
};

use http_common::{Method, Request, Response, ResponseError};

use crate::ReadState;

pub async fn serve_impl<
    MatchOutput: Future<Output = bool> + 'static,
    RespondOutput: Future<Output = Result<Response, ResponseError>> + 'static,
>(
    incoming_address: SocketAddr,
    mut response_stream: Arc<Async<TcpStream>>,
    match_func: impl Fn(String) -> MatchOutput,
    respond_func: impl Fn((SocketAddr, Request)) -> RespondOutput,
) {
    let mut method: Option<Method> = None;
    let mut uri: Option<String> = None;
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

                    if match_func(key.clone()).await {
                        // info!("incoming request matched url: {}", key);
                        read_state = ReadState::ReadingHeaders;
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

                        if let Some(method) = method.clone() {
                            if method == Method::Get {
                                read_state = ReadState::Finished;
                                info!("GET req has no body to read. finished.");
                                break;
                            }
                        }
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
                    return;
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

    let response_result = respond_func((incoming_address, request)).await;

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

            response_stream_flush(response_stream).await;

            // info!("response sent");
        }
        Err(e) => {
            warn!("error when responding: {:?}", e.to_string());
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
    response_stream_flush(response_stream).await;
}

async fn response_stream_flush(mut response_stream: Arc<Async<TcpStream>>) {
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
