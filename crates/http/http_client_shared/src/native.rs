use std::collections::BTreeMap;

use http_common::{Request, RequestOptions, Response, ResponseError};

use async_channel::{Receiver, Sender};

/// Only available when compiling for native.
///
/// NOTE: `Ok(…)` is returned on network error.
/// `Err` is only for failure to use the fetch API.
pub fn fetch_blocking(
    request: &Request,
    request_options_opt: Option<RequestOptions>,
) -> crate::Result<Response> {
    let mut req = ureq::request(request.method.as_str(), &request.url);

    if let Some(request_options) = request_options_opt {
        if let Some(timeout_duration) = request_options.timeout_opt {
            req = req.timeout(timeout_duration);
        }
    }

    for header in &request.headers {
        req = req.set(header.0, header.1);
    }

    let resp = req.send_bytes(&request.body);

    let (ok, resp) = match resp {
        Ok(resp) => (true, resp),
        Err(ureq::Error::Status(_, resp)) => (false, resp), // Still read the body on e.g. 404
        Err(ureq::Error::Transport(error)) => {
            return Err(ResponseError::HttpError(error.to_string()))
        }
    };

    let url = resp.get_url().to_owned();
    let status = resp.status();
    let status_text = resp.status_text().to_owned();
    let mut headers = BTreeMap::new();
    for key in &resp.headers_names() {
        if let Some(value) = resp.header(key) {
            // lowercase for easy lookup
            headers.insert(key.to_ascii_lowercase(), value.to_owned());
        }
    }

    let mut reader = resp.into_reader();
    let mut body = vec![];
    use std::io::Read;
    reader
        .read_to_end(&mut body)
        .map_err(|err| ResponseError::HttpError(err.to_string()))?;

    let response = Response {
        url,
        ok,
        status,
        status_text,
        body,
        headers,
    };
    Ok(response)
}

// ----------------------------------------------------------------------------

pub(crate) fn fetch(
    request: Request,
    request_options_opt: Option<RequestOptions>,
    on_done: Box<dyn FnOnce(crate::Result<Response>) + Send>,
) {
    std::thread::Builder::new()
        .name("ehttp".to_owned())
        .spawn(move || on_done(fetch_blocking(&request, request_options_opt)))
        .expect("Failed to spawn ehttp thread");
}

pub(crate) async fn fetch_async(
    request: Request,
    request_options_opt: Option<RequestOptions>,
) -> crate::Result<Response> {
    let (tx, rx): (
        Sender<crate::Result<Response>>,
        Receiver<crate::Result<Response>>,
    ) = async_channel::bounded(1);

    fetch(
        request,
        request_options_opt,
        Box::new(move |received| tx.send_blocking(received).unwrap()),
    );
    rx.recv()
        .await
        .map_err(|err| err.to_string())
        .map_err(|estr| ResponseError::HttpError(estr))?
}
