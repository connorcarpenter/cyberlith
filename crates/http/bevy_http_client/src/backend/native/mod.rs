use executor::{self, smol::future, Task};

use http_common::{Request, RequestOptions, Response, ResponseError};

pub(crate) struct RequestTask(pub(crate) Task<Result<Response, ResponseError>>);

pub(crate) fn send_request(
    request: Request,
    request_options_opt: Option<RequestOptions>,
) -> RequestTask {

    let task = if let Some(request_options) = request_options_opt {
        executor::spawn(async {
            http_client_shared::fetch_async_with_options(request, request_options).await
        })
    } else {
        executor::spawn(async { http_client_shared::fetch_async(request).await })
    };

    RequestTask(task)
}

pub(crate) fn poll_task(task: &mut RequestTask) -> Option<Result<Response, ResponseError>> {
    match future::block_on(future::poll_once(&mut task.0)) {
        Some(Ok(response)) => Some(Ok(response)),
        Some(Err(error)) => Some(Err(error)),
        None => None,
    }
}
