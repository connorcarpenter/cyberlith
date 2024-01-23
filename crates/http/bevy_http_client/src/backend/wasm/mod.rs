use bevy_tasks::AsyncComputeTaskPool;
use crossbeam_channel::{bounded, Receiver};
use ehttp::Response as EhttpResponse;

use http_common::{Request, Response, ResponseError};

use crate::convert::request_http_to_ehttp;

pub(crate) struct RequestTask(pub Receiver<Result<EhttpResponse, ehttp::Error>>);

pub(crate) fn send_request(request: Request) -> RequestTask {
    let thread_pool = AsyncComputeTaskPool::get();

    let ereq = request_http_to_ehttp(request).unwrap();

    let (tx, task) = bounded(1);
    thread_pool
        .spawn(async move {
            let response = ehttp::fetch_async(ereq).await;
            tx.send(response).ok();
        })
        .detach();

    RequestTask(task)
}

pub(crate) fn poll_task(task: &mut RequestTask) -> Option<Result<Response, ResponseError>> {
    match task.0.try_recv() {
        Ok(Ok(ehttp_response)) => {
            let Ok(response) = crate::convert::response_ehttp_to_http(ehttp_response) else {
                return Some(Err(ResponseError::SerdeError));
            };
            Some(Ok(response))
        }
        Ok(Err(error)) => Some(Err(ResponseError::EhttpError(error))),
        Err(_) => None,
    }
}
