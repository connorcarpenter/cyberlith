use crate::tasks::{
    create_dir::{CreateDirResult, CreateDirTask},
    delete::{DeleteResult, DeleteTask},
    read::{ReadResult, ReadTask},
    read_dir::{ReadDirResult, ReadDirTask},
    write::{WriteResult, WriteTask},
};

pub enum FsTaskEnum {
    Read(ReadTask),
    Write(WriteTask),
    Delete(DeleteTask),
    ReadDir(ReadDirTask),
    CreateDir(CreateDirTask),
}

pub enum FsTaskResultEnum {
    Read(ReadResult),
    Write(WriteResult),
    Delete(DeleteResult),
    ReadDir(ReadDirResult),
    CreateDir(CreateDirResult),
}
