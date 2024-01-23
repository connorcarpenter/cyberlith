use bevy_tasks::{AsyncComputeTaskPool, Task};
use ehttp::Response as EhttpResponse;
use futures_lite::future;

use http_common::{Request, Response, ResponseError};

use crate::convert::request_http_to_ehttp;

pub(crate) struct RequestTask(pub(crate) Task<Result<EhttpResponse, ehttp::Error>>);

pub(crate) fn send_request(request: Request) -> RequestTask {
    let thread_pool = AsyncComputeTaskPool::get();

    let ereq = request_http_to_ehttp(request).unwrap();

    let task = thread_pool.spawn(async { ehttp::fetch_async(ereq).await });

    RequestTask(task)
}

pub(crate) fn poll_task(task: &mut RequestTask) -> Option<Result<Response, ResponseError>> {
    match future::block_on(future::poll_once(&mut task.0)) {
        Some(Ok(ehttp_response)) => {
            let Ok(response) = crate::convert::response_ehttp_to_http(ehttp_response) else {
                return Some(Err(ResponseError::SerdeError));
            };
            Some(Ok(response))
        }
        Some(Err(error)) => Some(Err(ResponseError::EhttpError(error))),
        None => None,
    }
}
