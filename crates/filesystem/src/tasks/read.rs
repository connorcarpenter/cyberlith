use std::path::PathBuf;

use crate::{error::TaskError, traits::{FsTask, FsTaskResult}};
use crate::tasks::task_enum::{FsTaskEnum, FsTaskResultEnum};

// Task
pub struct ReadTask {
    pub path: PathBuf,
}

impl ReadTask {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self {
            path: path.into(),
        }
    }
}

// Result
pub struct ReadResult {
    pub bytes: Vec<u8>,
}

impl ReadResult {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes
        }
    }
}

// Traits
impl FsTask for ReadTask {
    type Result = ReadResult;

    fn to_enum(self) -> FsTaskEnum {
        FsTaskEnum::Read(self)
    }

    fn from_enum(task_enum: FsTaskEnum) -> Result<Self, ()> {
        let FsTaskEnum::Read(task) = task_enum else {
            return Err(());
        };
        Ok(task)
    }
}

impl FsTaskResult for ReadResult {
    fn to_enum(self) -> FsTaskResultEnum {
        FsTaskResultEnum::Read(self)
    }

    fn from_enum(result_enum: FsTaskResultEnum) -> Result<Self, TaskError> {
        let FsTaskResultEnum::Read(result) = result_enum else {
            return Err(TaskError::InvalidResult);
        };
        Ok(result)
    }
}