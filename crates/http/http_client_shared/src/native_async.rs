
use url::Url;
use executor::smol::{channel, channel::{Receiver, Sender}, net::TcpStream, io::{AsyncWriteExt, AsyncReadExt}};

use http_common::{Request, RequestOptions, Response, ResponseError};

pub(crate) async fn fetch_async(
    request: Request,
    request_options_opt: Option<RequestOptions>,
) -> Result<Response, ResponseError> {

    let (sender, receiver): (
        Sender<Result<Response, ResponseError>>,
        Receiver<Result<Response, ResponseError>>,
    ) = channel::bounded(1);

    executor::spawn(async move {
        let result = fetch_async_inner(request).await;
        let _ = sender.send(result).await;

    }).detach();

    match receiver.recv().await {
        Ok(response) => response,
        Err(e) => Err(ResponseError::NetworkError(e.to_string())),
    }
}

async fn fetch_async_inner(request: Request,) -> Result<Response, ResponseError> {
    let url = Url::parse(&request.url).map_err(|e| ResponseError::NetworkError(e.to_string()))?;

    let mut tcp_stream = connect(&url).await?;

    let mut request_string = format!(
        "{} {} HTTP/1.1\r\n",
        request.method.as_str(),
        url.path(),
    );

    // Add the Host header
    if let Some(host) = url.host_str() {
        request_string.push_str(&format!("Host: {}\r\n", host));
    }

    // Add the other headers
    for (key, values) in request.headers_iter() {
        for value in values {
            request_string.push_str(&format!("{}: {}\r\n", key, value));
        }
    }

    // Add a blank line to indicate the end of headers
    request_string.push_str("\r\n");

    // info!("Sending Request: {}", request_string);

    // Write the request to the stream
    tcp_stream.write_all(request_string.as_bytes()).await.map_err(|e| ResponseError::NetworkError(e.to_string()))?;

    // Write the request body if there is one
    if !request.body.is_empty() {
        tcp_stream.write_all(&request.body).await.map_err(|e| ResponseError::NetworkError(e.to_string()))?;
    }

    // Get Response
    read_response(&request.url, &mut tcp_stream).await
}

async fn connect(url: &Url) -> Result<TcpStream, ResponseError> {
    let host = url.host_str().ok_or(ResponseError::NetworkError("invalid host".to_string()))?;
    let port = url.port_or_known_default().ok_or(ResponseError::NetworkError("invalid port".to_string()))?;
    let addr = format!("{}:{}", host, port);

    TcpStream::connect(&addr).await.map_err(|e| ResponseError::NetworkError(e.to_string()))
}

async fn read_response(
    request_url: &str,
    tcp_stream: &mut TcpStream,
) -> Result<Response, ResponseError> {
    let mut buffer = Vec::new();
    tcp_stream.read_to_end(&mut buffer).await.map_err(|e| ResponseError::NetworkError(e.to_string()))?;

    // Parse the response string into a Response object (this would depend on how your Response is structured)
    let response = parse_response(request_url, &buffer)?;

    Ok(response)
}

fn parse_response(request_url: &str, response_bytes: &[u8]) -> Result<Response, ResponseError> {
    let response_str = String::from_utf8_lossy(response_bytes);

    let (status_line, headers_str, body_start_index) = split_response(&response_str)?;
    let (status_code, status_text) = parse_status_line(status_line)?;
    let headers = parse_headers(headers_str)?;
    let body = response_bytes[body_start_index..].to_vec();

    let ok = status_code >= 200 && status_code < 300;

    let mut response = Response::default();
    response.url = request_url.to_string();
    response.ok = ok;
    response.status = status_code;
    response.status_text = status_text;
    for (key, value) in headers {
        response.insert_header(&key, &value);
    }
    response.body = body;

    Ok(response)
}

fn split_response(response_str: &str) -> Result<(&str, &str, usize), ResponseError> {
    let mut parts = response_str.splitn(3, "\r\n\r\n");
    let status_and_headers = parts.next().ok_or(ResponseError::NetworkError("Missing status and headers".to_string()))?;
    let mut status_and_headers_parts = status_and_headers.splitn(2, "\r\n");
    let status_line = status_and_headers_parts.next().ok_or(ResponseError::NetworkError("Missing status line".to_string()))?;
    let headers = status_and_headers_parts.next().unwrap_or(""); // If there are no headers, it's an empty string
    let body_start_index = response_str.find("\r\n\r\n").map(|idx| idx + 4).unwrap_or(response_str.len());
    Ok((status_line, headers, body_start_index))
}

fn parse_status_line(status_line: &str) -> Result<(u16, String), ResponseError> {
    let mut parts = status_line.splitn(3, ' ');
    let _http_version = parts.next().ok_or(ResponseError::NetworkError("Missing HTTP version".to_string()))?;
    let status_code = parts.next().ok_or(ResponseError::NetworkError("Missing status code".to_string()))?;
    let status_text = parts.next().unwrap_or("").to_string(); // Status text can be empty
    let status_code = status_code.parse::<u16>().map_err(|e| ResponseError::NetworkError(format!("Invalid status code: {}", e)))?;
    Ok((status_code, status_text))
}

fn parse_headers(headers: &str) -> Result<Vec<(String, String)>, ResponseError> {
    let mut header_store: Vec<(String, String)> = Vec::new();
    for line in headers.lines() {
        let mut parts = line.splitn(2, ": ");
        let key = parts.next().ok_or(ResponseError::NetworkError("Missing header key".to_string()))?.to_lowercase();
        let value = parts.next().ok_or(ResponseError::NetworkError("Missing header value".to_string()))?.to_string();
        header_store.push((key, value));
    }
    Ok(header_store)
}