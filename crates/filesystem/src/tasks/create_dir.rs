use std::path::PathBuf;

use crate::{
    error::TaskError,
    tasks::{
        task_enum::{FsTaskEnum, FsTaskResultEnum},
        traits::{FsTask, FsTaskResult},
    },
};

// Task
pub struct CreateDirTask {
    pub path: PathBuf,
}

impl CreateDirTask {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self { path: path.into() }
    }
}

// Result
pub struct CreateDirResult;

impl CreateDirResult {
    pub fn new() -> Self {
        Self
    }
}

// Traits
impl FsTask for CreateDirTask {
    type Result = CreateDirResult;

    fn to_enum(self) -> FsTaskEnum {
        FsTaskEnum::CreateDir(self)
    }

    // fn from_enum(task_enum: FsTaskEnum) -> Result<Self, ()> {
    //     let FsTaskEnum::CreateDir(task) = task_enum else {
    //         return Err(());
    //     };
    //     Ok(task)
    // }
}

impl FsTaskResult for CreateDirResult {
    fn to_enum(self) -> FsTaskResultEnum {
        FsTaskResultEnum::CreateDir(self)
    }

    fn from_enum(result_enum: FsTaskResultEnum) -> Result<Self, TaskError> {
        let FsTaskResultEnum::CreateDir(result) = result_enum else {
            return Err(TaskError::InvalidResult);
        };
        Ok(result)
    }
}
