//! Minimal HTTP client for both native and WASM.

mod types;

use http_common::{Request, RequestOptions, Response, ResponseError};

#[cfg(not(target_arch = "wasm32"))]
mod native_async;

/// Performs an `async` HTTP request.
pub async fn fetch_async(request: Request) -> Result<Response, ResponseError> {
    #[cfg(not(target_arch = "wasm32"))]
    return native_async::fetch_async(request, None).await;

    #[cfg(target_arch = "wasm32")]
    return web::fetch_async(&request, None).await;
}

/// Performs an `async` HTTP request.
pub async fn fetch_async_with_options(
    request: Request,
    request_options: RequestOptions,
) -> Result<Response, ResponseError> {
    #[cfg(not(target_arch = "wasm32"))]
    return native_async::fetch_async(request, Some(request_options)).await;

    #[cfg(target_arch = "wasm32")]
    return web::fetch_async(&request, Some(request_options)).await;
}

#[cfg(not(target_arch = "wasm32"))]
mod native_blocking;
#[cfg(not(target_arch = "wasm32"))]
pub use native_blocking::fetch_blocking;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::spawn_future;

/// Helper for constructing [`Request::headers`].
pub fn headers(headers: &[(&str, &str)]) -> std::collections::BTreeMap<String, String> {
    headers
        .iter()
        .map(|e| (e.0.to_owned(), e.1.to_owned()))
        .collect()
}
