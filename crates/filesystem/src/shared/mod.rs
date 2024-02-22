//! Minimal HTTP client for both native and WASM.

use crate::{common::{FsTaskEnum, FsTaskResultEnum}, FsTaskError};

pub(crate) async fn fetch_async(task_enum: FsTaskEnum) -> Result<FsTaskResultEnum, FsTaskError> {
    #[cfg(not(target_arch = "wasm32"))]
    return native::fetch_async(task_enum).await;

    #[cfg(target_arch = "wasm32")]
    return web::fetch_async(&task_enum).await;
}

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod web;