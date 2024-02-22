use bevy_tasks::{AsyncComputeTaskPool, Task};

use futures_lite::future;

use async_channel::{Receiver, Sender};

use crate::{task_write::WriteResult, task_read::ReadResult, error::FsTaskError, types::{FsTaskEnum, FsTaskResultEnum}};

pub(crate) struct FsTaskJob(pub(crate) Task<Result<FsTaskResultEnum, FsTaskError>>);

pub(crate) fn start_task(
    task_enum: FsTaskEnum,
) -> FsTaskJob {
    let thread_pool = AsyncComputeTaskPool::get();

    let task = thread_pool.spawn(async { crate::backend::fetch_async(task_enum).await });

    FsTaskJob(task)
}

pub(crate) fn poll_task(task: &mut FsTaskJob) -> Option<Result<FsTaskResultEnum, FsTaskError>> {
    match future::block_on(future::poll_once(&mut task.0)) {
        Some(Ok(result_enum)) => Some(Ok(result_enum)),
        Some(Err(error)) => Some(Err(error)),
        None => None,
    }
}

////




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