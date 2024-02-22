use std::collections::HashMap;

use bevy_ecs::{change_detection::ResMut, system::Resource};

use crate::common::{FsTask, FsTaskResult, ReadTask, ReadResult, FsTaskResultEnum, FsTaskError, WriteTask, WriteResult};

use crate::{
    backend::FsTaskJob,
    backend::{poll_task, start_task},
    TaskKey,
};

#[derive(Resource)]
pub struct FileSystemClient {
    tasks: HashMap<u64, FsTaskJob>,
    results: HashMap<u64, Result<FsTaskResultEnum, FsTaskError>>,
    current_index: u64,
}

impl Default for FileSystemClient {
    fn default() -> Self {
        Self {
            tasks: HashMap::new(),
            results: HashMap::new(),
            current_index: 0,
        }
    }
}

impl FileSystemClient {
    fn start_task<Q: FsTask>(
        &mut self,
        task: Q,
    ) -> TaskKey<Q::Result> {
        let task_enum = task.to_enum();

        let rtask = start_task(task_enum);

        let key = self.next_key();

        self.tasks.insert(key.id, rtask);

        key
    }

    pub fn read(
        &mut self,
        path: &str,
    ) -> TaskKey<ReadResult> {
        self.start_task(ReadTask::new(path))
    }

    pub fn write(
        &mut self,
        path: &str,
        bytes: Vec<u8>,
    ) -> TaskKey<WriteResult> {
        self.start_task(WriteTask::new(path, bytes))
    }

    pub fn get_result<S: FsTaskResult>(
        &mut self,
        key: &TaskKey<S>,
    ) -> Option<Result<S, FsTaskError>> {
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

    pub(crate) fn accept_result(&mut self, key: u64, result: Result<FsTaskResultEnum, FsTaskError>) {
        self.tasks.remove(&key);
        self.results.insert(key, result);
    }
}

pub(crate) fn client_update(mut client: ResMut<FileSystemClient>) {
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
