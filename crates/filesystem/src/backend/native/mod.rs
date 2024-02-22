use bevy_tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

use crate::common::{FsTaskEnum, FsTaskResultEnum, FsTaskError};

pub(crate) struct FsTaskJob(pub(crate) Task<Result<FsTaskResultEnum, FsTaskError>>);

pub(crate) fn start_task(
    task_enum: FsTaskEnum,
) -> FsTaskJob {
    let thread_pool = AsyncComputeTaskPool::get();

    let task = thread_pool.spawn(async { crate::shared::fetch_async(task_enum).await });

    FsTaskJob(task)
}

pub(crate) fn poll_task(task: &mut FsTaskJob) -> Option<Result<FsTaskResultEnum, FsTaskError>> {
    match future::block_on(future::poll_once(&mut task.0)) {
        Some(Ok(result_enum)) => Some(Ok(result_enum)),
        Some(Err(error)) => Some(Err(error)),
        None => None,
    }
}
