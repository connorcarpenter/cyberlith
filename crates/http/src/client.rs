use std::collections::HashMap;

use bevy_ecs::{change_detection::ResMut, system::Resource};

use crate::{HttpKey, HttpRequest, backend::RequestTask, HttpResponse, HttpResponseError, backend::{send_request, poll_task}};

#[derive(Resource)]
pub struct HttpClient {
    tasks: HashMap<HttpKey, RequestTask>,
    results: HashMap<HttpKey, Result<HttpResponse, HttpResponseError>>,
    current_index: u64,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            tasks: HashMap::new(),
            results: HashMap::new(),
            current_index: 0,
        }
    }
}

impl HttpClient {

    pub fn send(&mut self, request: HttpRequest) -> HttpKey {
        let task = send_request(request);

        let key = self.next_key();

        self.tasks.insert(key, task);

        key
    }

    pub fn recv(&mut self, key: &HttpKey) -> Option<Result<HttpResponse, HttpResponseError>> {
        if let Some(result) = self.results.remove(key) {
            return Some(result);
        } else {
            return None;
        }
    }

    pub fn recv_all(&mut self) -> Vec<(HttpKey, Result<HttpResponse, HttpResponseError>)> {
        let mut results = Vec::new();
        for (key, result) in self.results.drain() {
            results.push((key, result));
        }
        results
    }

    fn next_key(&mut self) -> HttpKey {
        let old_index = self.current_index;
        self.current_index = self.current_index.wrapping_add(1);
        HttpKey(old_index)
    }

    pub(crate) fn tasks_iter_mut(&mut self) -> impl Iterator<Item = (&HttpKey, &mut RequestTask)> {
        self.tasks.iter_mut()
    }

    pub(crate) fn accept_result(&mut self, key: HttpKey, result: Result<HttpResponse, HttpResponseError>) {
        self.tasks.remove(&key);
        self.results.insert(key, result);
    }
}

pub(crate) fn client_update(
    mut client: ResMut<HttpClient>,
) {
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