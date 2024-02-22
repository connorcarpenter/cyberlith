
use crate::common::{ReadTask, ReadResult, WriteTask, WriteResult};

pub enum FsTaskEnum {
    Read(ReadTask),
    Write(WriteTask),
}

pub enum FsTaskResultEnum {
    Read(ReadResult),
    Write(WriteResult),
}