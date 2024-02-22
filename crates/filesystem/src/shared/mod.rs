//! Minimal HTTP client for both native and WASM.

use crate::common::{Request, RequestOptions, Response};

/// Performs an HTTP request and calls the given callback when done.
// pub fn fetch(request: Request, on_done: impl 'static + Send + FnOnce(Result<Response>)) {
//     #[cfg(not(target_arch = "wasm32"))]
//     native::fetch(request, Box::new(on_done));
//
//     #[cfg(target_arch = "wasm32")]
//     web::fetch(request, Box::new(on_done));
// }

/// Performs an `async` HTTP request.
pub async fn fetch_async(request: Request) -> Result<Response> {
    #[cfg(not(target_arch = "wasm32"))]
    return native::fetch_async(request, None).await;

    #[cfg(target_arch = "wasm32")]
    return web::fetch_async(&request, None).await;
}

/// Performs an `async` HTTP request.
pub async fn fetch_async_with_options(
    request: Request,
    request_options: RequestOptions,
) -> Result<Response> {
    #[cfg(not(target_arch = "wasm32"))]
    return native::fetch_async(request, Some(request_options)).await;

    #[cfg(target_arch = "wasm32")]
    return web::fetch_async(&request, Some(request_options)).await;
}

mod types;
pub use types::Result;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod web;