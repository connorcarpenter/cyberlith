use crate::tasks::{
    create_dir::{CreateDirResult, CreateDirTask},
    read::{ReadResult, ReadTask},
    read_dir::{ReadDirResult, ReadDirTask},
    write::{WriteResult, WriteTask},
};

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
