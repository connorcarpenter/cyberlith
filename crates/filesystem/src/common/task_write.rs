use std::path::PathBuf;

use crate::common::{FsTask, FsTaskResult, FsTaskEnum, FsTaskResultEnum};
use crate::FsTaskError;

// Task
pub struct WriteTask {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

impl WriteTask {
    pub fn new<T: Into<PathBuf>>(path: T, bytes: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            bytes,
        }
    }
}

// Result
pub struct WriteResult;

impl WriteResult {
    pub fn new() -> Self {
        Self
    }
}

// Traits
impl FsTask for WriteTask {
    type Result = WriteResult;

    fn to_enum(self) -> FsTaskEnum {
        FsTaskEnum::Write(self)
    }

    fn from_enum(task_enum: FsTaskEnum) -> Result<Self, ()> {
        let FsTaskEnum::Write(task) = task_enum else {
            return Err(());
        };
        Ok(task)
    }
}

impl FsTaskResult for WriteResult {
    fn to_enum(self) -> FsTaskResultEnum {
        FsTaskResultEnum::Write(self)
    }

    fn from_enum(result_enum: FsTaskResultEnum) -> Result<Self, FsTaskError> {
        let FsTaskResultEnum::Write(result) = result_enum else {
            return Err(FsTaskError::InvalidResult);
        };
        Ok(result)
    }
}