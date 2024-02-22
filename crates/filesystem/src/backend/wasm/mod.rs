use bevy_tasks::AsyncComputeTaskPool;

use crossbeam_channel::{bounded, Receiver};

use crate::{tasks::task_enum::{FsTaskEnum, FsTaskResultEnum}, error::TaskError};

pub(crate) struct FsTaskJob(pub Receiver<Result<FsTaskResultEnum, TaskError>>);

pub(crate) fn start_task(
    task_enum: FsTaskEnum,
) -> FsTaskJob {
    let thread_pool = AsyncComputeTaskPool::get();

    let (tx, task) = bounded(1);
    thread_pool
        .spawn(async move {
            let result = crate::backend::task_process_async(task_enum).await;
            tx.send(result).ok();
        })
        .detach();

    FsTaskJob(task)
}

pub(crate) fn poll_task(task: &mut FsTaskJob) -> Option<Result<FsTaskResultEnum, TaskError>> {
    match task.0.try_recv() {
        Ok(Ok(result_enum)) => Some(Ok(result_enum)),
        Ok(Err(error)) => Some(Err(error)),
        Err(_) => None,
    }
}

pub async fn task_process_async(
    _task_enum: &FsTaskEnum,
) -> Result<FsTaskResultEnum, TaskError> {
    todo!()
}