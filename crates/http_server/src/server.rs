use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    pin::Pin,
    task::{Context, Poll},
};

use async_dup::Arc;
use http::{header, HeaderValue, Response};
use log::info;
use smol::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    prelude::*,
    Async,
};

use crate::executor;

pub fn start_server(
    socket_addr: SocketAddr,
) {
    executor::spawn(async move {
        listen(socket_addr).await;
    })
    .detach();
}

/// Listens for incoming connections and serves them.
async fn listen(
    socket_addr: SocketAddr,
) {
    let listener = Async::<TcpListener>::bind(socket_addr)
        .expect("unable to bind a TCP Listener to the supplied socket address");
    info!(
        "Listening at http://{}/",
        listener
            .get_ref()
            .local_addr()
            .expect("Listener does not have a local address"),
    );

    loop {
        // Accept the next connection.
        let (response_stream, _) = listener
            .accept()
            .await
            .expect("was not able to accept the incoming stream from the listener");

        // Spawn a background task serving this connection.
        executor::spawn(async move {
            serve(Arc::new(response_stream)).await;
        })
        .detach();
    }
}

/// Reads a request from the client and sends it a response.
async fn serve(mut stream: Arc<Async<TcpStream>>) {
    info!("serving");

    let remote_addr = stream
        .get_ref()
        .local_addr()
        .expect("stream does not have a local address");
    let mut success: bool = false;
    let mut headers_read: bool = false;
    let mut content_length: Option<usize> = None;
    let mut rtc_url_match = false;
    let mut body: Vec<u8> = Vec::new();

    let buf_reader = BufReader::new(stream.clone());
    let mut bytes = buf_reader.bytes();
    {
        let mut line: Vec<u8> = Vec::new();
        while let Some(byte) = bytes.next().await {
            let byte = byte.expect("unable to read a byte from incoming stream");

            if headers_read {
                if let Some(content_length) = content_length {
                    body.push(byte);

                    if body.len() >= content_length {
                        success = true;
                        break;
                    }
                } else {
                    info!("request was missing Content-Length header");
                    break;
                }
            }

            if byte == b'\r' {
                continue;
            } else if byte == b'\n' {
                let mut str = String::from_utf8(line.clone())
                    .expect("unable to parse string from UTF-8 bytes");
                line.clear();

                if rtc_url_match {
                    if str.to_lowercase().starts_with("content-length: ") {
                        let (_, last) = str.split_at(16);
                        str = last.to_string();
                        content_length = str.parse::<usize>().ok();
                    } else if str.is_empty() {
                        headers_read = true;
                    }
                } else if str.starts_with(
                    "some_random_path"
                ) {
                    rtc_url_match = true;
                }
            } else {
                line.push(byte);
            }
        }

        if success {
            success = false;

            let mut lines = body.lines();

            info!("sending out request");
            match outwards_request(&mut lines).await {
                Ok(mut response) => {
                    info!("received out request");
                    success = true;

                    response.headers_mut().insert(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        HeaderValue::from_static("*"),
                    );

                    let mut out = response_header_to_vec(&response);
                    out.extend_from_slice(response.body().as_bytes());

                    info!("Successful  request from {}", remote_addr);

                    stream
                        .write_all(&out)
                        .await
                        .expect("found an error while writing to a stream");
                }
                Err(err) => {
                    info!(
                        "Invalid request from {}",
                        remote_addr,
                    );
                }
            }
        }
    }

    if !success {
        stream.write_all(RESPONSE_BAD).await.expect("found");
    }

    stream.flush().await.expect("unable to flush the stream");
    stream.close().await.expect("unable to close the stream");
}

const RESPONSE_BAD: &[u8] = br#"
HTTP/1.1 404 NOT FOUND
Content-Type: text/html
Content-Length: 0
Access-Control-Allow-Origin: *
"#;

struct RequestBuffer<'a, R: AsyncBufRead + Unpin> {
    buffer: &'a mut Lines<R>,
    add_newline: bool,
}

impl<'a, R: AsyncBufRead + Unpin> RequestBuffer<'a, R> {
    fn new(buf: &'a mut Lines<R>) -> Self {
        RequestBuffer {
            add_newline: false,
            buffer: buf,
        }
    }
}

type ReqError = std::io::Error;

const NEWLINE_STR: &str = "\n";

async fn outwards_request<R: AsyncBufRead + Unpin>(lines: &mut Lines<R>) -> Result<Response<String>, ()> {
    info!("in out request");
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body("some_body".to_string())
        .expect("could not construct session response"))
}

fn response_header_to_vec<T>(r: &Response<T>) -> Vec<u8> {
    let v = Vec::with_capacity(120);
    let mut c = std::io::Cursor::new(v);
    write_response_header(r, &mut c).expect("unable to write response header to stream");
    c.into_inner()
}

fn write_response_header<T>(
    r: &Response<T>,
    mut io: impl std::io::Write,
) -> std::io::Result<usize> {
    let mut len = 0;
    macro_rules! w {
        ($x:expr) => {
            io.write_all($x)?;
            len += $x.len();
        };
    }

    let status = r.status();
    let code = status.as_str();
    let reason = status.canonical_reason().unwrap_or("Unknown");
    let headers = r.headers();

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

fn write_line(io: &mut dyn std::io::Write, len: &mut usize, mut buf: &[u8]) -> std::io::Result<()> {
    io.write_all(buf)?;
    *len += buf.len();
    Ok(())
}