use std::collections::HashMap;

use bevy_ecs::{change_detection::ResMut, system::Resource};

use http_common::{ApiRequest, ApiResponse, Request, RequestOptions, Response, ResponseError};

use crate::{
    backend::RequestTask,
    backend::{poll_task, send_request},
    ResponseKey,
};

#[derive(Resource)]
pub struct HttpClient {
    tasks: HashMap<u64, RequestTask>,
    results: HashMap<u64, Result<Response, ResponseError>>,
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
    pub fn send<Q: ApiRequest>(
        &mut self,
        addr: &str,
        port: u16,
        req: Q,
    ) -> ResponseKey<Q::Response> {
        let url = if port == 443 {
            format!("https://{}/{}", addr, Q::path())
        } else {
            format!("http://{}:{}/{}", addr, port, Q::path())
        };
        let http_request = Request::new(Q::method(), &url, req.to_bytes().to_vec());
        //info!("Sending request to: {:?}", url);

        let task = send_request(http_request, None);

        let key = self.next_key();

        self.tasks.insert(key.id, task);

        key
    }

    pub fn send_with_options<Q: ApiRequest>(
        &mut self,
        addr: &str,
        port: u16,
        req: Q,
        req_options: RequestOptions,
    ) -> ResponseKey<Q::Response> {
        let url = format!("http://{}:{}/{}", addr, port, Q::path());
        let http_request = Request::new(Q::method(), &url, req.to_bytes().to_vec());
        //info!("Sending request to: {:?}", url);

        let task = send_request(http_request, Some(req_options));

        let key = self.next_key();

        self.tasks.insert(key.id, task);

        key
    }

    pub fn recv<S: ApiResponse>(
        &mut self,
        key: &ResponseKey<S>,
    ) -> Option<Result<S, ResponseError>> {
        if let Some(result) = self.results.remove(&key.id) {
            match result {
                Ok(response) => {
                    let Ok(api_response) = S::from_response(response) else {
                        return Some(Err(ResponseError::SerdeError));
                    };
                    Some(Ok(api_response))
                }
                Err(err) => Some(Err(err)),
            }
        } else {
            return None;
        }
    }

    fn next_key<S: ApiResponse>(&mut self) -> ResponseKey<S> {
        let next_index = self.current_index;
        self.current_index = self.current_index.wrapping_add(1);
        ResponseKey::new(next_index)
    }

    pub(crate) fn tasks_iter_mut(&mut self) -> impl Iterator<Item = (&u64, &mut RequestTask)> {
        self.tasks.iter_mut()
    }

    pub(crate) fn accept_result(&mut self, key: u64, result: Result<Response, ResponseError>) {
        self.tasks.remove(&key);
        self.results.insert(key, result);
    }
}

pub(crate) fn client_update(mut client: ResMut<HttpClient>) {
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
