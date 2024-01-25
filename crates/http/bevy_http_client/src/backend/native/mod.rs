use bevy_tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

use http_common::{Request, Response, ResponseError};

pub(crate) struct RequestTask(pub(crate) Task<Result<Response, ResponseError>>);

pub(crate) fn send_request(request: Request) -> RequestTask {
    let thread_pool = AsyncComputeTaskPool::get();

    let task = thread_pool.spawn(async { http_client_shared::fetch_async(request).await });

    RequestTask(task)
}

pub(crate) fn poll_task(task: &mut RequestTask) -> Option<Result<Response, ResponseError>> {
    match future::block_on(future::poll_once(&mut task.0)) {
        Some(Ok(response)) => {
            Some(Ok(response))
        }
        Some(Err(error)) => Some(Err(error)),
        None => None,
    }
}
