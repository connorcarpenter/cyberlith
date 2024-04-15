use std::collections::HashMap;

use bevy_ecs::{change_detection::ResMut, system::Resource};

use crate::{ClipboardManagerImpl, error::TaskError, native::{poll_task, start_task, TaskJob}, task_key::TaskKey};

#[derive(Resource)]
pub struct ClipboardManager {
    pub(crate) inner: ClipboardManagerImpl,

    tasks: HashMap<u64, TaskJob>,
    results: HashMap<u64, Result<String, TaskError>>,
    current_index: u64,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self {
            inner: ClipboardManagerImpl::default(),

            tasks: HashMap::new(),
            results: HashMap::new(),
            current_index: 0,
        }
    }
}

impl ClipboardManager {
    /// Sets clipboard contents.
    pub fn set_contents(&mut self, contents: &str) {
        self.inner.set_contents(contents);
    }

    /// Gets clipboard contents. Returns [`None`] if clipboard provider is unavailable or returns an error.
    pub fn get_contents(&mut self) -> TaskKey {
        self.start_task()
    }

    pub fn get_result(
        &mut self,
        key: &TaskKey,
    ) -> Option<Result<String, TaskError>> {
        if let Some(result) = self.results.remove(&key.id) {
            match result {
                Ok(result) => {
                    return Some(Ok(result));
                }
                Err(err) => Some(Err(err)),
            }
        } else {
            return None;
        }
    }

    fn start_task(&mut self) -> TaskKey {

        let job = start_task();

        let key = self.next_key();

        self.tasks.insert(key.id, job);

        key
    }

    fn next_key(&mut self) -> TaskKey {
        let next_index = self.current_index;
        self.current_index = self.current_index.wrapping_add(1);
        TaskKey::new(next_index)
    }

    pub(crate) fn tasks_iter_mut(&mut self) -> impl Iterator<Item = (&u64, &mut TaskJob)> {
        self.tasks.iter_mut()
    }

    pub(crate) fn accept_result(&mut self, key: u64, result: Result<String, TaskError>) {
        self.tasks.remove(&key);
        self.results.insert(key, result);
    }
}

// used as a system
pub(crate) fn update(mut client: ResMut<ClipboardManager>) {
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