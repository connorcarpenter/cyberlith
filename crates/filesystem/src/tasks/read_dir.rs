use std::path::PathBuf;

use crate::{error::TaskError, tasks::task_enum::{FsTaskEnum, FsTaskResultEnum}, tasks::traits::{FsTask, FsTaskResult}};

// Task
pub struct ReadDirTask {
    pub path: PathBuf,
}

impl ReadDirTask {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self {
            path: path.into(),
        }
    }
}

// Result
pub struct ReadDirEntry {
    path: PathBuf,
    file_name: String,
}

impl ReadDirEntry {
    pub fn new(path: PathBuf, file_name: String) -> Self {
        Self {
            path,
            file_name,
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}

pub struct ReadDirResult {
    entries: Vec<ReadDirEntry>
}

impl ReadDirResult {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: ReadDirEntry) {
        self.entries.push(entry);
    }

    pub fn entries(self) -> Vec<ReadDirEntry> {
        self.entries
    }
}

// Traits
impl FsTask for ReadDirTask {
    type Result = ReadDirResult;

    fn to_enum(self) -> FsTaskEnum {
        FsTaskEnum::ReadDir(self)
    }

    fn from_enum(task_enum: FsTaskEnum) -> Result<Self, ()> {
        let FsTaskEnum::ReadDir(task) = task_enum else {
            return Err(());
        };
        Ok(task)
    }
}

impl FsTaskResult for ReadDirResult {
    fn to_enum(self) -> FsTaskResultEnum {
        FsTaskResultEnum::ReadDir(self)
    }

    fn from_enum(result_enum: FsTaskResultEnum) -> Result<Self, TaskError> {
        let FsTaskResultEnum::ReadDir(result) = result_enum else {
            return Err(TaskError::InvalidResult);
        };
        Ok(result)
    }
}