
use crate::common::{ReadResponse, Request, Response, ResponseError, WriteResponse};

use async_channel::{Receiver, Sender};

pub(crate) async fn fetch_async(
    request: Request,
) -> Result<Response, ResponseError> {
    let (tx, rx): (
        Sender<Result<Response, ResponseError>>,
        Receiver<Result<Response, ResponseError>>,
    ) = async_channel::bounded(1);

    fetch(
        request,
        Box::new(move |received| tx.send_blocking(received).unwrap()),
    );
    rx.recv()
        .await
        .map_err(|err| ResponseError::IoError(err.to_string()))?
}

fn fetch(
    request: Request,
    on_done: Box<dyn FnOnce(Result<Response, ResponseError>) + Send>,
) {
    std::thread::Builder::new()
        .name("filesystem_client".to_owned())
        .spawn(move || on_done(fetch_blocking(&request)))
        .expect("Failed to spawn ehttp thread");
}

fn fetch_blocking(
    request: &Request,
) -> Result<Response, ResponseError> {
    match request {
        Request::Read(request) => {
            match std::fs::read(&request.path) {
                Ok(bytes) => {
                    Ok(Response::Read(ReadResponse::new(bytes)))
                }
                Err(e) => {
                    Err(ResponseError::IoError(e.to_string()))
                }
            }
        }
        Request::Write(request) => {
            match std::fs::write(&request.path, &request.bytes) {
                Ok(()) => {
                    Ok(Response::Write(WriteResponse::new()))
                }
                Err(e) => {
                    Err(ResponseError::IoError(e.to_string()))
                }
            }
        }
    }
}
