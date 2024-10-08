use http_common::{Request, RequestOptions, Response, ResponseError};

/// Only available when compiling for native.
///
/// NOTE: `Ok(…)` is returned on network error.
/// `Err` is only for failure to use the fetch API.
pub fn fetch_blocking(
    request: &Request,
    request_options_opt: Option<RequestOptions>,
) -> Result<Response, ResponseError> {
    let builder = ureq::builder().redirects(0); // we handle redirects automatically
    let agent = builder.build();
    let mut req = agent.request(request.method.as_str(), &request.url);

    if let Some(request_options) = request_options_opt {
        if let Some(timeout_duration) = request_options.timeout_opt {
            req = req.timeout(timeout_duration);
        }
    }

    // info!("Sending Request with Headers:");
    for (header_name, header_values) in request.headers_iter() {
        for header_value in header_values {
            // info!("[{}:{}]", header_name, header_value);
            req = req.set(header_name, header_value);
        }
    }
    // info!("---");

    let resp = req.send_bytes(&request.body);

    let (ok, resp) = match resp {
        Ok(resp) => (true, resp),
        Err(ureq::Error::Status(_, resp)) => (false, resp), // Still read the body on e.g. 404
        Err(ureq::Error::Transport(error)) => {
            return Err(ResponseError::NetworkError(error.to_string()))
        }
    };

    let url = resp.get_url().to_owned();
    let status = resp.status();
    let status_text = resp.status_text().to_owned();
    let mut headers = Vec::new();
    for key in &resp.headers_names() {
        for value in resp.all(key) {
            // lowercase for easy lookup
            headers.push((key.to_ascii_lowercase(), value.to_owned()));
        }
    }

    let mut reader = resp.into_reader();
    let mut body = vec![];
    use std::io::Read;
    reader
        .read_to_end(&mut body)
        .map_err(|err| ResponseError::NetworkError(err.to_string()))?;

    let mut response = Response::default();
    response.url = url;
    response.ok = ok;
    response.status = status;
    response.status_text = status_text;
    response.body = body;
    for (header_name, header_value) in headers {
        response.insert_header(&header_name, &header_value);
    }
    Ok(response)
}
