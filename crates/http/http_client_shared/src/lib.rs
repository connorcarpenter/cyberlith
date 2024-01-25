//! Minimal HTTP client for both native and WASM.

use http_common::{Request, Response};

/// Performs an HTTP request and calls the given callback when done.
pub fn fetch(request: Request, on_done: impl 'static + Send + FnOnce(Result<Response>)) {
    #[cfg(not(target_arch = "wasm32"))]
    native::fetch(request, Box::new(on_done));

    #[cfg(target_arch = "wasm32")]
    web::fetch(request, Box::new(on_done));
}

/// Performs an `async` HTTP request.
pub async fn fetch_async(request: Request) -> Result<Response> {
    #[cfg(not(target_arch = "wasm32"))]
    return native::fetch_async(request).await;

    #[cfg(target_arch = "wasm32")]
    return web::fetch_async(&request).await;
}

mod types;
pub use types::Result;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub use native::fetch_blocking;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::spawn_future;

/// Helper for constructing [`Request::headers`].
/// ```
/// use http_common::Request;
/// let request = Request {
///     headers: http_client_shared::headers(&[
///         ("Accept", "*/*"),
///         ("Content-Type", "text/plain; charset=utf-8"),
///     ]),
///     ..Request::get("https://www.example.com")
/// };
/// ```
pub fn headers(headers: &[(&str, &str)]) -> std::collections::BTreeMap<String, String> {
    headers
        .iter()
        .map(|e| (e.0.to_owned(), e.1.to_owned()))
        .collect()
}
