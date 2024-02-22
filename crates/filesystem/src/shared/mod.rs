//! Minimal HTTP client for both native and WASM.

use crate::common::{Request, Response};
use crate::ResponseError;

/// Performs an `async` HTTP request.
pub async fn fetch_async(request: Request) -> Result<Response, ResponseError> {
    #[cfg(not(target_arch = "wasm32"))]
    return native::fetch_async(request).await;

    #[cfg(target_arch = "wasm32")]
    return web::fetch_async(&request).await;
}

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod web;