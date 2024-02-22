use crate::tasks::read::{ReadResult, ReadTask};
use crate::tasks::write::{WriteResult, WriteTask};

pub enum FsTaskEnum {
    Read(ReadTask),
    Write(WriteTask),
}

pub enum FsTaskResultEnum {
    Read(ReadResult),
    Write(WriteResult),
}