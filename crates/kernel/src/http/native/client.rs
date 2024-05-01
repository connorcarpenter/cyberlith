use bevy_ecs::system::{ResMut, Resource};

use bevy_http_client::{HttpClient as InnerHttpClient, ResponseError, ResponseKey};
use http_common::{ApiRequest, ApiResponse, RequestOptions};

#[derive(Resource)]
pub struct HttpClient {
    inner: InnerHttpClient,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            inner: InnerHttpClient::default(),
        }
    }
}

impl HttpClient {
    pub(crate) fn update_system(mut client: ResMut<Self>) {
        InnerHttpClient::update(&mut client.inner);
    }

    pub fn send<Q: ApiRequest>(
        &mut self,
        addr: &str,
        port: u16,
        req: Q,
    ) -> ResponseKey<Q::Response> {
        self.inner.send(addr, port, req)
    }

    pub fn send_with_options<Q: ApiRequest>(
        &mut self,
        addr: &str,
        port: u16,
        req: Q,
        req_options: RequestOptions,
    ) -> ResponseKey<Q::Response> {
        self.inner.send_with_options(addr, port, req, req_options)
    }

    pub fn recv<S: ApiResponse>(
        &mut self,
        key: &ResponseKey<S>,
    ) -> Option<Result<S, ResponseError>> {
        self.inner.recv(key)
    }
}