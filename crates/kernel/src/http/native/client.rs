use std::sync::{Arc, RwLock};

use bevy_ecs::system::{ResMut, Resource};

use bevy_http_client::{HttpClient as InnerHttpClient, ResponseError, ResponseKey};
use http_common::{ApiRequest, ApiResponse, RequestOptions, Response};

use crate::http::native::cookie_store::CookieStore;

#[derive(Resource)]
pub struct HttpClient {
    inner: InnerHttpClient,
    cookie_store: Arc<RwLock<CookieStore>>,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            inner: InnerHttpClient::default(),
            cookie_store: Arc::new(RwLock::new(CookieStore::new())),
        }
    }
}

impl HttpClient {
    pub(crate) fn update_system(mut client: ResMut<Self>) {
        let cookie_store_clone = client.cookie_store.clone();
        InnerHttpClient::update(
            &mut client.inner,
            move |response| {
                let mut cookie_store = cookie_store_clone.write().unwrap();
                cookie_store.handle_response(response);
            }
        );
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