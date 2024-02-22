use crate::{task_read::{ReadTask, ReadResult}, task_write::{WriteResult, WriteTask}};

pub enum FsTaskEnum {
    Read(ReadTask),
    Write(WriteTask),
}

pub enum FsTaskResultEnum {
    Read(ReadResult),
    Write(WriteResult),
}