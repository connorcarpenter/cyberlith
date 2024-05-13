use std::path::PathBuf;

use crate::error::TaskError;
use crate::tasks::task_enum::{FsTaskEnum, FsTaskResultEnum};
use crate::tasks::traits::{FsTask, FsTaskResult};

// Task
pub struct WriteTask {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
}

impl WriteTask {
    pub fn new<T: Into<PathBuf>, C: AsRef<[u8]>>(path: T, bytes: C) -> Self {
        let my_bytes = bytes.as_ref().to_vec();
        Self {
            path: path.into(),
            bytes: my_bytes,
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

    // fn from_enum(task_enum: FsTaskEnum) -> Result<Self, ()> {
    //     let FsTaskEnum::Write(task) = task_enum else {
    //         return Err(());
    //     };
    //     Ok(task)
    // }
}

impl FsTaskResult for WriteResult {
    fn to_enum(self) -> FsTaskResultEnum {
        FsTaskResultEnum::Write(self)
    }

    fn from_enum(result_enum: FsTaskResultEnum) -> Result<Self, TaskError> {
        let FsTaskResultEnum::Write(result) = result_enum else {
            return Err(TaskError::InvalidResult);
        };
        Ok(result)
    }
}
