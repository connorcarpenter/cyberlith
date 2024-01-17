
use bevy_tasks::{AsyncComputeTaskPool, Task};
use ehttp::Response;
use futures_lite::future;

use crate::{HttpRequest, HttpResponse, HttpResponseError};

pub(crate) struct RequestTask(pub(crate) Task<Result<Response, ehttp::Error>>);

pub(crate) fn send_request(
    request: HttpRequest,
) -> RequestTask {
    let thread_pool = AsyncComputeTaskPool::get();

    let inner_request = request.0;

    let task = thread_pool.spawn(async { ehttp::fetch_async(inner_request).await });

    RequestTask(task)
}

pub(crate) fn poll_task(
    task: &mut RequestTask,
) -> Option<Result<HttpResponse, HttpResponseError>> {
    match future::block_on(future::poll_once(&mut task.0)) {
        Some(Ok(response)) => Some(Ok(HttpResponse(response))),
        Some(Err(error)) => Some(Err(HttpResponseError(error))),
        None => None,
    }
}
