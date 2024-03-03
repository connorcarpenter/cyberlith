use bevy_tasks::{AsyncComputeTaskPool, Task};

use futures_lite::future;

use async_channel::{Receiver, Sender};
use log::info;

use crate::{
    error::TaskError,
    tasks::{
        read::ReadResult,
        read_dir::ReadDirEntry,
        task_enum::{FsTaskEnum, FsTaskResultEnum},
        write::WriteResult,
    },
    CreateDirResult, ReadDirResult,
};

pub(crate) struct FsTaskJob(pub(crate) Task<Result<FsTaskResultEnum, TaskError>>);

pub(crate) fn start_task(task_enum: FsTaskEnum) -> FsTaskJob {
    let thread_pool = AsyncComputeTaskPool::get();

    let task = thread_pool.spawn(async { crate::backend::task_process_async(task_enum).await });

    FsTaskJob(task)
}

pub(crate) fn poll_task(task: &mut FsTaskJob) -> Option<Result<FsTaskResultEnum, TaskError>> {
    match future::block_on(future::poll_once(&mut task.0)) {
        Some(Ok(result_enum)) => Some(Ok(result_enum)),
        Some(Err(error)) => Some(Err(error)),
        None => None,
    }
}

////

pub(crate) async fn task_process_async(
    task_enum: FsTaskEnum,
) -> Result<FsTaskResultEnum, TaskError> {
    let (tx, rx): (
        Sender<Result<FsTaskResultEnum, TaskError>>,
        Receiver<Result<FsTaskResultEnum, TaskError>>,
    ) = async_channel::bounded(1);

    task_process(
        task_enum,
        Box::new(move |received| tx.send_blocking(received).unwrap()),
    );
    rx.recv()
        .await
        .map_err(|err| TaskError::IoError(err.to_string()))?
}

fn task_process(
    task_enum: FsTaskEnum,
    on_done: Box<dyn FnOnce(Result<FsTaskResultEnum, TaskError>) + Send>,
) {
    std::thread::Builder::new()
        .name("filesystem".to_owned())
        .spawn(move || on_done(task_process_blocking(&task_enum)))
        .expect("Failed to spawn filesystem thread");
}

fn task_process_blocking(task_enum: &FsTaskEnum) -> Result<FsTaskResultEnum, TaskError> {
    match task_enum {
        FsTaskEnum::Read(task) => match std::fs::read(&task.path) {
            Ok(bytes) => Ok(FsTaskResultEnum::Read(ReadResult::new(bytes))),
            Err(e) => Err(TaskError::IoError(e.to_string())),
        },
        FsTaskEnum::Write(task) => match std::fs::write(&task.path, &task.bytes) {
            Ok(()) => Ok(FsTaskResultEnum::Write(WriteResult::new())),
            Err(e) => Err(TaskError::IoError(e.to_string())),
        },
        FsTaskEnum::ReadDir(task) => match std::fs::read_dir(&task.path) {
            Ok(entries) => Ok(FsTaskResultEnum::ReadDir(convert_read_dir(entries))),
            Err(e) => Err(TaskError::IoError(e.to_string())),
        },
        FsTaskEnum::CreateDir(task) => match std::fs::create_dir(&task.path) {
            Ok(()) => Ok(FsTaskResultEnum::CreateDir(CreateDirResult::new())),
            Err(e) => Err(TaskError::IoError(e.to_string())),
        },
    }
}

fn convert_read_dir(read_dir: std::fs::ReadDir) -> ReadDirResult {
    let mut result = ReadDirResult::new();

    for entry in read_dir {
        match entry {
            Ok(entry) => {
                let result_entry = convert_dir_entry(entry);
                result.add_entry(result_entry);
            }
            Err(e) => {
                // skip for now
                info!("Error reading directory entry: {:?}", e.to_string());
            }
        }
    }

    result
}

fn convert_dir_entry(entry: std::fs::DirEntry) -> ReadDirEntry {
    let path = entry.path();
    let file_name = entry.file_name().into_string().unwrap();

    ReadDirEntry::new(path, file_name)
}
