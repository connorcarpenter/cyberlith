use bevy_tasks::AsyncComputeTaskPool;

use crossbeam_channel::{bounded, Receiver};

use crate::{tasks::{write::WriteTask, read_dir::ReadDirTask, read::ReadTask, create_dir::CreateDirTask, task_enum::{FsTaskEnum, FsTaskResultEnum}}, error::TaskError};

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
    task_enum: &FsTaskEnum,
) -> Result<FsTaskResultEnum, TaskError> {
    match task_enum {
        FsTaskEnum::Read(task) => handle_read(task).await,
        FsTaskEnum::Write(task) => handle_write(task).await,
        FsTaskEnum::ReadDir(task) => handle_read_dir(task).await,
        FsTaskEnum::CreateDir(task) => handle_create_dir(task).await,
    }
}

async fn handle_read(task: &ReadTask) -> Result<FsTaskResultEnum, TaskError> {
    todo!()
}

async fn handle_write(task: &WriteTask) -> Result<FsTaskResultEnum, TaskError> {
    todo!()
}

async fn handle_read_dir(task: &ReadDirTask) -> Result<FsTaskResultEnum, TaskError> {
    todo!()
}

async fn handle_create_dir(task: &CreateDirTask) -> Result<FsTaskResultEnum, TaskError> {
    todo!()
}