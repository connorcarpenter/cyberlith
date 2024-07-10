use executor::AsyncComputeTaskPool;
use crossbeam_channel::{bounded, Receiver};

use http_common::{Request, RequestOptions, Response, ResponseError};

pub(crate) struct RequestTask(pub Receiver<Result<Response, ResponseError>>);

pub(crate) fn send_request(
    request: Request,
    request_options_opt: Option<RequestOptions>,
) -> RequestTask {
    let thread_pool = AsyncComputeTaskPool::get();

    let (tx, task) = bounded(1);
    thread_pool
        .spawn(async move {
            let response = if let Some(request_options) = request_options_opt {
                http_client_shared::fetch_async_with_options(request, request_options).await
            } else {
                http_client_shared::fetch_async(request).await
            };
            tx.send(response).ok();
        })
        .detach();

    RequestTask(task)
}

pub(crate) fn poll_task(task: &mut RequestTask) -> Option<Result<Response, ResponseError>> {
    match task.0.try_recv() {
        Ok(Ok(response)) => Some(Ok(response)),
        Ok(Err(error)) => Some(Err(error)),
        Err(_) => None,
    }
}
