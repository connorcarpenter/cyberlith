
use bevy_tasks::AsyncComputeTaskPool;
use ehttp::Response;
use crossbeam_channel::{bounded, Receiver};

use crate::{HttpRequest, HttpResponse, HttpResponseError};

pub(crate) struct RequestTask(pub Receiver<Result<Response, ehttp::Error>>);

pub(crate) fn send_request(
    request: HttpRequest,
) -> RequestTask {
    let thread_pool = AsyncComputeTaskPool::get();

    let inner_request = request.0;

    let (tx, task) = bounded(1);
    thread_pool
        .spawn(async move {
            let response = ehttp::fetch_async(inner_request).await;
            tx.send(response).ok();
        })
        .detach();

    RequestTask(task)
}

pub(crate) fn poll_task(
    task: &mut RequestTask,
) -> Option<Result<HttpResponse, HttpResponseError>> {
    match task.0.try_recv() {
        Ok(Ok(response)) => Some(Ok(HttpResponse(response))),
        Ok(Err(error)) => Some(Err(HttpResponseError(error))),
        Err(_) => None,
    }
}