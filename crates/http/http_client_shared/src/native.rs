
use http_common::{Request, RequestOptions, Response, ResponseError};

use async_channel::{Receiver, Sender};

/// Only available when compiling for native.
///
/// NOTE: `Ok(…)` is returned on network error.
/// `Err` is only for failure to use the fetch API.
pub fn fetch_blocking(
    request: &Request,
    request_options_opt: Option<RequestOptions>,
) -> Result<Response, ResponseError> {
    let mut req = ureq::request(request.method.as_str(), &request.url);

    if let Some(request_options) = request_options_opt {
        if let Some(timeout_duration) = request_options.timeout_opt {
            req = req.timeout(timeout_duration);
        }
    }

    for (header_name, header_value) in request.headers_iter() {
        req = req.set(header_name, header_value);
    }

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
        if let Some(value) = resp.header(key) {
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
        response.set_header(&header_name, &header_value);
    }
    Ok(response)
}

// ----------------------------------------------------------------------------

pub(crate) fn fetch(
    request: Request,
    request_options_opt: Option<RequestOptions>,
    on_done: Box<dyn FnOnce(Result<Response, ResponseError>) + Send>,
) {
    std::thread::Builder::new()
        .name("ehttp".to_owned())
        .spawn(move || on_done(fetch_blocking(&request, request_options_opt)))
        .expect("Failed to spawn ehttp thread");
}

pub(crate) async fn fetch_async(
    request: Request,
    request_options_opt: Option<RequestOptions>,
) -> Result<Response, ResponseError> {
    let (tx, rx): (
        Sender<Result<Response, ResponseError>>,
        Receiver<Result<Response, ResponseError>>,
    ) = async_channel::bounded(1);

    fetch(
        request,
        request_options_opt,
        Box::new(move |received| tx.send_blocking(received).unwrap()),
    );
    rx.recv()
        .await
        .map_err(|err| err.to_string())
        .map_err(|estr| ResponseError::NetworkError(estr))?
}
