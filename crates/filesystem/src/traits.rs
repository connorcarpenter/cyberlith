
use crate::{types::{FsTaskEnum, FsTaskResultEnum}, error::FsTaskError};

pub trait FsTask {
    type Result: FsTaskResult;

    fn to_enum(self) -> FsTaskEnum;
    fn from_enum(task_enum: FsTaskEnum) -> Result<Self, ()> where Self: Sized;
}

pub trait FsTaskResult {
    fn to_enum(self) -> FsTaskResultEnum;
    fn from_enum(result_enum: FsTaskResultEnum) -> Result<Self, FsTaskError> where Self: Sized;
}
