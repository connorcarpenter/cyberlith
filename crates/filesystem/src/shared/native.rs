
use crate::common::{ReadResult, FsTaskEnum, FsTaskResultEnum, FsTaskError, WriteResult};

use async_channel::{Receiver, Sender};

pub(crate) async fn fetch_async(
    task_enum: FsTaskEnum,
) -> Result<FsTaskResultEnum, FsTaskError> {
    let (tx, rx): (
        Sender<Result<FsTaskResultEnum, FsTaskError>>,
        Receiver<Result<FsTaskResultEnum, FsTaskError>>,
    ) = async_channel::bounded(1);

    fetch(
        task_enum,
        Box::new(move |received| tx.send_blocking(received).unwrap()),
    );
    rx.recv()
        .await
        .map_err(|err| FsTaskError::IoError(err.to_string()))?
}

fn fetch(
    task_enum: FsTaskEnum,
    on_done: Box<dyn FnOnce(Result<FsTaskResultEnum, FsTaskError>) + Send>,
) {
    std::thread::Builder::new()
        .name("filesystem_client".to_owned())
        .spawn(move || on_done(fetch_blocking(&task_enum)))
        .expect("Failed to spawn ehttp thread");
}

fn fetch_blocking(
    task_enum: &FsTaskEnum,
) -> Result<FsTaskResultEnum, FsTaskError> {
    match task_enum {
        FsTaskEnum::Read(task) => {
            match std::fs::read(&task.path) {
                Ok(bytes) => {
                    Ok(FsTaskResultEnum::Read(ReadResult::new(bytes)))
                }
                Err(e) => {
                    Err(FsTaskError::IoError(e.to_string()))
                }
            }
        }
        FsTaskEnum::Write(task) => {
            match std::fs::write(&task.path, &task.bytes) {
                Ok(()) => {
                    Ok(FsTaskResultEnum::Write(WriteResult::new()))
                }
                Err(e) => {
                    Err(FsTaskError::IoError(e.to_string()))
                }
            }
        }
    }
}
