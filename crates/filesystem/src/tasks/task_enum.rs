use crate::tasks::{write::{WriteResult, WriteTask}, read_dir::{ReadDirResult, ReadDirTask}, read::{ReadResult, ReadTask}, create_dir::{CreateDirTask, CreateDirResult}};

pub enum FsTaskEnum {
    Read(ReadTask),
    Write(WriteTask),
    ReadDir(ReadDirTask),
    CreateDir(CreateDirTask),
}

pub enum FsTaskResultEnum {
    Read(ReadResult),
    Write(WriteResult),
    ReadDir(ReadDirResult),
    CreateDir(CreateDirResult),
}