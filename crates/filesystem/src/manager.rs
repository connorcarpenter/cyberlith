use std::{path::PathBuf, collections::HashMap};

use bevy_ecs::{change_detection::ResMut, system::Resource};

use crate::{
    backend::{FsTaskJob, poll_task, start_task},
    error::TaskError,
    TaskKey,
    tasks::{write::{WriteResult, WriteTask}, traits::{FsTask, FsTaskResult}, task_enum::FsTaskResultEnum, read::{ReadResult, ReadTask}},
};

#[derive(Resource)]
pub struct FileSystemManager {
    tasks: HashMap<u64, FsTaskJob>,
    results: HashMap<u64, Result<FsTaskResultEnum, TaskError>>,
    current_index: u64,
}

impl Default for FileSystemManager {
    fn default() -> Self {
        Self {
            tasks: HashMap::new(),
            results: HashMap::new(),
            current_index: 0,
        }
    }
}

impl FileSystemManager {
    fn start_task<Q: FsTask>(
        &mut self,
        task: Q,
    ) -> TaskKey<Q::Result> {
        let task_enum = task.to_enum();

        let job = start_task(task_enum);

        let key = self.next_key();

        self.tasks.insert(key.id, job);

        key
    }

    pub fn read<T: Into<PathBuf>>(&mut self, path: T) -> TaskKey<ReadResult> {
        self.start_task(ReadTask::new(path))
    }

    pub fn write<T: Into<PathBuf>, C: AsRef<[u8]>>(&mut self, path: T, bytes: C) -> TaskKey<WriteResult> {
        self.start_task(WriteTask::new(path, bytes))
    }

    pub fn get_result<S: FsTaskResult>(
        &mut self,
        key: &TaskKey<S>,
    ) -> Option<Result<S, TaskError>> {
        if let Some(result) = self.results.remove(&key.id) {
            match result {
                Ok(result_enum) => {
                    return Some(S::from_enum(result_enum));
                }
                Err(err) => Some(Err(err)),
            }
        } else {
            return None;
        }
    }

    fn next_key<S: FsTaskResult>(&mut self) -> TaskKey<S> {
        let next_index = self.current_index;
        self.current_index = self.current_index.wrapping_add(1);
        TaskKey::new(next_index)
    }

    pub(crate) fn tasks_iter_mut(&mut self) -> impl Iterator<Item = (&u64, &mut FsTaskJob)> {
        self.tasks.iter_mut()
    }

    pub(crate) fn accept_result(&mut self, key: u64, result: Result<FsTaskResultEnum, TaskError>) {
        self.tasks.remove(&key);
        self.results.insert(key, result);
    }
}

pub(crate) fn update(mut client: ResMut<FileSystemManager>) {
    let mut finished_tasks = Vec::new();
    for (key, task) in client.tasks_iter_mut() {
        if let Some(result) = poll_task(task) {
            finished_tasks.push((*key, result));
        }
    }
    for (key, result) in finished_tasks {
        client.accept_result(key, result);
    }
}
