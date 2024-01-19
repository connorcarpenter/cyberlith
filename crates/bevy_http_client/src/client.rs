use std::collections::HashMap;

use bevy_ecs::{change_detection::ResMut, system::Resource};

use crate::{ResponseKey, HttpRequest, backend::RequestTask, HttpResponse, HttpResponseError, backend::{send_request, poll_task}, ClientHttpRequest, ClientHttpResponse};

#[derive(Resource)]
pub struct HttpClient {
    tasks: HashMap<u64, RequestTask>,
    results: HashMap<u64, Result<HttpResponse, HttpResponseError>>,
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

    pub fn send<Q: ClientHttpRequest>(&mut self, url: &str, req: Q) -> ResponseKey<Q::Response> {

        let http_request = HttpRequest::post(url, req.to_bytes());

        let task = send_request(http_request);

        let key = self.next_key();

        self.tasks.insert(key.id, task);

        key
    }

    pub fn recv<S: ClientHttpResponse>(&mut self, key: &ResponseKey<S>) -> Option<Result<S, HttpResponseError>> {
        if let Some(result) = self.results.remove(&key.id) {
            match result {
                Ok(response) => {
                    Some(Ok(S::from(response)))
                }
                Err(err) => Some(Err(err)),
            }
        } else {
            return None;
        }
    }

    fn next_key<S: ClientHttpResponse>(&mut self) -> ResponseKey<S> {
        let next_index = self.current_index;
        self.current_index = self.current_index.wrapping_add(1);
        ResponseKey::new(next_index)
    }

    pub(crate) fn tasks_iter_mut(&mut self) -> impl Iterator<Item = (&u64, &mut RequestTask)> {
        self.tasks.iter_mut()
    }

    pub(crate) fn accept_result(&mut self, key: u64, result: Result<HttpResponse, HttpResponseError>) {
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