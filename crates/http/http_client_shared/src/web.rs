use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::AbortController;

use http_common::{Request, RequestOptions, Response, ResponseError};
// use log::info;

use crate::types::PartialResponse;

/// Only available when compiling for web.
///
/// NOTE: `Ok(…)` is returned on network error.
/// `Err` is only for failure to use the fetch API.
pub async fn fetch_async(
    request: &Request,
    request_options_opt: Option<RequestOptions>,
) -> Result<Response, ResponseError> {
    fetch_jsvalue(request, request_options_opt)
        .await
        .map_err(string_from_js_value)
        .map_err(|estr| ResponseError::NetworkError(estr))
}

pub(crate) fn string_from_js_value(value: JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{:#?}", value))
}

pub(crate) async fn fetch_base(
    request: &Request,
    request_options_opt: Option<RequestOptions>,
) -> Result<web_sys::Response, JsValue> {
    let opts = web_sys::RequestInit::new();
    opts.set_method(request.method.as_str());
    opts.set_mode(web_sys::RequestMode::Cors);
    if let Some(request_options) = request_options_opt {
        if let Some(timeout) = request_options.timeout_opt {
            let abort_controller = AbortController::new()?;
            let abort_signal = abort_controller.signal();

            // abort the controller after a timeout
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    Closure::wrap(Box::new(move || {
                        abort_controller.abort();
                    }) as Box<dyn FnMut()>)
                    .as_ref()
                    .unchecked_ref(),
                    timeout.as_millis() as i32,
                )
                .expect("cannot set timeout");

            opts.set_signal(Some(&abort_signal));
        }
    }

    if !request.body.is_empty() {
        let body_bytes: &[u8] = &request.body;
        let body_array: js_sys::Uint8Array = body_bytes.into();
        let js_value: &JsValue = body_array.as_ref();
        opts.set_body(js_value);
    }

    let js_request = web_sys::Request::new_with_str_and_init(&request.url, &opts)?;

    for (name, values) in request.headers_iter() {
        for value in values {
            // info!("Setting header: {}={}", name, value);
            js_request.headers().set(name, value)?;
        }
    }

    let window = web_sys::window().unwrap();
    let response = JsFuture::from(window.fetch_with_request(&js_request)).await?;
    let response: web_sys::Response = response.dyn_into()?;

    Ok(response)
}

pub(crate) fn get_response_base(response: &web_sys::Response) -> Result<PartialResponse, JsValue> {
    // https://developer.mozilla.org/en-US/docs/Web/API/Headers
    // "Note: When Header values are iterated over, […] values from duplicate header names are combined."
    let js_headers: web_sys::Headers = response.headers();
    let js_iter = js_sys::try_iter(&js_headers)
        .expect("headers try_iter")
        .expect("headers have an iterator");

    let mut headers = std::collections::BTreeMap::new();
    for item in js_iter {
        let item = item.expect("headers iterator");
        let array: js_sys::Array = item.into();
        let v: Vec<JsValue> = array.to_vec();

        let mut key = v[0]
            .as_string()
            .ok_or_else(|| JsValue::from_str("headers name"))?;
        let value = v[1]
            .as_string()
            .ok_or_else(|| JsValue::from_str("headers value"))?;

        // for easy lookup
        key.make_ascii_lowercase();
        headers.insert(key, value);
    }

    Ok(PartialResponse {
        url: response.url(),
        ok: response.ok(),
        status: response.status(),
        status_text: response.status_text(),
        headers,
    })
}

/// NOTE: `Ok(…)` is returned on network error.
/// `Err` is only for failure to use the fetch API.
async fn fetch_jsvalue(
    request: &Request,
    request_options_opt: Option<RequestOptions>,
) -> Result<Response, JsValue> {
    let response = fetch_base(request, request_options_opt).await?;

    let array_buffer = JsFuture::from(response.array_buffer()?).await?;
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let body = uint8_array.to_vec();

    let base = get_response_base(&response)?;

    let mut outgoing_response = Response::default();

    for (name, value) in base.headers {
        outgoing_response.insert_header(&name, &value);
    }

    outgoing_response.url = base.url;
    outgoing_response.ok = base.ok;
    outgoing_response.status = base.status;
    outgoing_response.status_text = base.status_text;
    outgoing_response.body = body;

    Ok(outgoing_response)
}

/// Spawn an async task.
///
/// A wrapper around `wasm_bindgen_futures::spawn_local`.
/// Only available with the web backend.
pub fn spawn_future<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

// ----------------------------------------------------------------------------

// pub(crate) fn fetch(request: Request, on_done: Box<dyn FnOnce(crate::Result<Response>) + Send>) {
//     spawn_future(async move {
//         let result = fetch_async(&request).await;
//         on_done(result)
//     });
// }
