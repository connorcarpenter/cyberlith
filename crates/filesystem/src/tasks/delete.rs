use std::path::PathBuf;

use crate::error::TaskError;
use crate::tasks::task_enum::{FsTaskEnum, FsTaskResultEnum};
use crate::tasks::traits::{FsTask, FsTaskResult};

// Task
pub struct DeleteTask {
    pub path: PathBuf,
}

impl DeleteTask {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self {
            path: path.into(),
        }
    }
}

// Result
pub struct DeleteResult;

impl DeleteResult {
    pub fn new() -> Self {
        Self
    }
}

// Traits
impl FsTask for DeleteTask {
    type Result = DeleteResult;

    fn to_enum(self) -> FsTaskEnum {
        FsTaskEnum::Delete(self)
    }

    // fn from_enum(task_enum: FsTaskEnum) -> Result<Self, ()> {
    //     let FsTaskEnum::Delete(task) = task_enum else {
    //         return Err(());
    //     };
    //     Ok(task)
    // }
}

impl FsTaskResult for DeleteResult {
    fn to_enum(self) -> FsTaskResultEnum {
        FsTaskResultEnum::Delete(self)
    }

    fn from_enum(result_enum: FsTaskResultEnum) -> Result<Self, TaskError> {
        let FsTaskResultEnum::Delete(result) = result_enum else {
            return Err(TaskError::InvalidResult);
        };
        Ok(result)
    }
}
