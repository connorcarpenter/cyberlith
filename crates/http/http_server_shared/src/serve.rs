use std::{collections::BTreeMap, net::SocketAddr};

use smol::{
    future::Future,
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    stream::StreamExt,
};

use http_common::{Method, Request, Response, ResponseError};
use logging::{info, warn};

use crate::ReadState;

pub enum MatchHostResult {
    Match,
    // includes redirect url
    NoMatchRedirect(String),
    // includes error message?
    NoMatch,
}

pub async fn serve_impl<
    MatchUrlOutput: Future<Output = bool> + 'static,
    MatchHostOutput: Future<Output = MatchHostResult> + 'static,
    ResponseOutput: Future<Output = Result<Response, ResponseError>> + 'static,
    ResponseStream: Unpin + AsyncRead + AsyncWrite,
>(
    incoming_address: SocketAddr,
    mut response_stream: ResponseStream,
    match_url_func: impl Fn(String) -> MatchUrlOutput,
    match_host_func: impl Fn(String, String) -> MatchHostOutput,
    response_func: impl Fn(String, SocketAddr, Request) -> ResponseOutput,
) {
    let mut method: Option<Method> = None;
    let mut uri: Option<String> = None;
    let mut endpoint_key: Option<String> = None;
    let mut content_length: Option<usize> = None;
    let mut body: Vec<u8> = Vec::new();
    let mut header_map = BTreeMap::<String, String>::new();

    let mut read_state = ReadState::MatchingUrl;

    let buf_reader = BufReader::new(&mut response_stream);
    let mut bytes = buf_reader.bytes();

    let mut line: Vec<u8> = Vec::new();

    loop {
        let Some(byte) = bytes.next().await else {
            info!("no more bytes!");
            break;
        };

        let byte = byte.expect("unable to read a byte from incoming stream");

        match read_state {
            ReadState::MatchingUrl => {
                if byte == b'\r' {
                    continue;
                } else if byte == b'\n' {
                    let line_str = String::from_utf8(line.clone())
                        .expect("unable to parse string from UTF-8 bytes");
                    line.clear();

                    //info!("read: {}", str);

                    let url_key = request_extract_url(&mut method, &mut uri, &line_str);

                    if !match_url_func(url_key.clone()).await {
                        warn!("no endpoint found for {}", url_key);
                        read_state = ReadState::Error;
                        break;
                    }

                    // info!("incoming request matched url: {}", key);
                    read_state = ReadState::ReadingHeaders;
                    endpoint_key = Some(url_key);
                    continue;
                } else {
                    line.push(byte);
                }
            }
            ReadState::ReadingHeaders => {
                let Some(endpoint_key) = endpoint_key.as_ref() else {
                    warn!("no endpoint key found");
                    read_state = ReadState::Error;
                    break;
                };
                if byte == b'\r' {
                    continue;
                } else if byte == b'\n' {
                    let line_str = String::from_utf8(line.clone())
                        .expect("unable to parse string from UTF-8 bytes");
                    line.clear();

                    //info!("read: {}", str);

                    if request_read_headers(
                        &match_host_func,
                        endpoint_key,
                        &mut method,
                        &mut content_length,
                        &mut header_map,
                        &mut read_state,
                        &line_str,
                    )
                    .await
                    {
                        break;
                    } else {
                        continue;
                    }
                } else {
                    line.push(byte);
                }
            }
            ReadState::ReadingBody => {
                if request_read_body(content_length, &mut body, &mut read_state, byte) {
                    break;
                } else {
                    continue;
                }
            }
            _ => {
                warn!("shouldn't be in this state");
                return;
            }
        }
    }

    let Some(incoming_url) = uri else {
        let response = Response::not_found("");
        return response_send(response_stream, response).await;
    };

    match read_state {
        ReadState::Finished => {
            // success! continue,
        }
        ReadState::Redirecting(redirect_url) => {
            let response = Response::redirect(&incoming_url, &redirect_url);
            return response_send(response_stream, response).await;
        }
        _ => {
            let response = Response::not_found(&incoming_url);
            return response_send(response_stream, response).await;
        }
    }

    // done reading //

    let Some(request) = cast_to_request(method, &incoming_url, body, header_map).await else {
        let response = Response::not_found(&incoming_url);
        return response_send(response_stream, response).await;
    };
    let Some(endpoint_key) = endpoint_key else {
        let response = Response::not_found(&incoming_url);
        return response_send(response_stream, response).await;
    };

    match response_func(endpoint_key, incoming_address, request.clone()).await {
        Ok(response) => {
            return response_send(response_stream, response).await;
        }
        Err(e) => {
            let response = e.to_response(&request.url);
            return response_send(response_stream, response).await;
        }
    }
}

fn request_extract_url(
    method: &mut Option<Method>,
    uri: &mut Option<String>,
    line_str: &String,
) -> String {
    let parts = line_str.split(" ").collect::<Vec<&str>>();
    let key = format!("{} {}", parts[0], parts[1]);
    *method = Some(Method::from_str(parts[0]).unwrap());
    *uri = Some(parts[1].to_string());
    key
}

async fn request_read_headers<MatchHostOutput: Future<Output = MatchHostResult> + 'static>(
    match_host_func: &impl Fn(String, String) -> MatchHostOutput,
    endpoint_key: &str,
    method: &mut Option<Method>,
    content_length: &mut Option<usize>,
    header_map: &mut BTreeMap<String, String>,
    read_state: &mut ReadState,
    line_str: &String,
) -> bool {
    if line_str.is_empty() {
        //info!("finished reading headers.");

        *read_state = ReadState::ReadingBody;

        // check for GET request, if so, we're done
        if let Some(method) = method.clone() {
            if method == Method::Get || method == Method::Options {
                *read_state = ReadState::Finished;
                // info!("GET req has no body to read. finished.");
                return true;
            }
        }

        // check for content-length header for non-Get requests
        let Some(content_length) = content_length else {
            warn!("request was missing Content-Length header");
            *read_state = ReadState::Error;
            return true;
        };
        if *content_length == 0 {
            *read_state = ReadState::Finished;
            //info!("no body to read. finished.");
            return true;
        } else {
            return false;
        }
    } else {
        let parts = line_str.split(": ").collect::<Vec<&str>>();

        // insert into header collection
        header_map.insert(parts[0].to_string(), parts[1].to_string());

        // check headers
        match parts[0].to_lowercase().as_str() {
            "host" => {
                // check if host is allowed
                let host = parts[1].to_string();
                let match_host_result = match_host_func(endpoint_key.to_string(), host).await;
                match match_host_result {
                    MatchHostResult::NoMatchRedirect(redirect_url) => {
                        warn!(
                            "host not allowed: {}, redirecting to {}",
                            parts[1], redirect_url
                        );
                        *read_state = ReadState::Redirecting(redirect_url);
                        return true;
                    }
                    MatchHostResult::NoMatch => {
                        warn!("host not allowed: {}", parts[1]);
                        *read_state = ReadState::Error;
                        return true;
                    }
                    MatchHostResult::Match => {}
                }
            }
            "content-length" => {
                // capture content-length header
                *content_length = Some(parts[1].parse().unwrap());
            }
            _ => {}
        }

        return false;
    }
}

fn request_read_body(
    content_length: Option<usize>,
    body: &mut Vec<u8>,
    read_state: &mut ReadState,
    byte: u8,
) -> bool {
    //info!("read byte from body");

    if let Some(content_length) = content_length {
        body.push(byte);

        if body.len() >= content_length {
            *read_state = ReadState::Finished;
            //info!("finished reading body");
            return true;
        }

        return false;
    } else {
        warn!("request was missing Content-Length header");
        *read_state = ReadState::Error;
        return true;
    }
}

async fn cast_to_request(
    method: Option<Method>,
    uri: &str,
    body: Vec<u8>,
    header_map: BTreeMap<String, String>,
) -> Option<Request> {
    // cast to request //
    let Some(method) = method else {
        warn!("unable to parse method");
        return None;
    };
    let mut request = Request::new(method, uri, body);
    for (hn, hv) in header_map {
        request.set_header(&hn, &hv);
    }
    Some(request)
}

async fn response_send<ResponseStream: Unpin + AsyncRead + AsyncWrite>(
    mut response_stream: ResponseStream,
    response: Response,
) {
    let mut response_bytes = response_header_to_vec(&response);
    response_bytes.extend_from_slice(&response.body);
    response_stream
        .write_all(&response_bytes)
        .await
        .expect("found an error while writing to a stream");

    response_stream_flush(response_stream).await;

    // info!("response sent");
}

//
// const RESPONSE_BAD: &[u8] = br#"
// HTTP/1.1 404 NOT FOUND
// Content-Type: text/html
// Content-Length: 0
// "#;
//
// async fn send_404<ResponseStream: Unpin + AsyncRead + AsyncWrite>(
//     mut response_stream: ResponseStream,
// ) {
//     let response = Response::not_found()
//     response_stream.write_all(RESPONSE_BAD).await.unwrap();
//     response_stream_flush(response_stream).await;
// }

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
    let reason = r.status_text.as_str();

    write_line(&mut io, &mut len, b"HTTP/1.1 ")?;
    write_line(&mut io, &mut len, code.as_bytes())?;
    write_line(&mut io, &mut len, b" ")?;
    write_line(&mut io, &mut len, reason.as_bytes())?;
    write_line(&mut io, &mut len, b"\r\n")?;

    for (hn, hv) in r.headers_iter() {
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

async fn response_stream_flush<ResponseStream: Unpin + AsyncRead + AsyncWrite>(
    mut response_stream: ResponseStream,
) {
    response_stream
        .flush()
        .await
        .expect("unable to flush the stream");

    response_stream
        .close()
        .await
        .expect("unable to close the stream");
}
