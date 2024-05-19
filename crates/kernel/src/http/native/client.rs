use std::sync::{Arc, RwLock};

use bevy_ecs::system::{ResMut, Resource};

use bevy_http_client::{HttpClient as InnerHttpClient, ResponseError, ResponseKey};
use http_common::{ApiRequest, ApiResponse};

use crate::http::native::cookie_store::CookieStore;

#[derive(Resource)]
pub struct HttpClient {
    inner: InnerHttpClient,
    cookie_store_opt: Option<Arc<RwLock<CookieStore>>>,
}

impl Default for HttpClient {
    fn default() -> Self {
        panic!("HttpClient::default() is not supported in native!")
    }
}

impl HttpClient {
    pub fn new(cookie_store_opt: Option<Arc<RwLock<CookieStore>>>) -> Self {
        Self {
            inner: InnerHttpClient::default(),
            cookie_store_opt,
        }
    }

    pub(crate) fn update_system(mut client: ResMut<Self>) {
        if let Some(cookie_store_clone) = client.cookie_store_opt.clone() {
            InnerHttpClient::update(&mut client.inner, |response| {
                let mut cookie_store = cookie_store_clone.write().unwrap();
                cookie_store.handle_response(response);
            })
        } else {
            InnerHttpClient::update(&mut client.inner, |_| {})
        }
    }

    pub fn send<Q: ApiRequest>(
        &mut self,
        addr: &str,
        port: u16,
        req: Q,
    ) -> ResponseKey<Q::Response> {
        if let Some(cookie_store_clone) = self.cookie_store_opt.clone() {
            self.inner.send_with_middleware(addr, port, req, |request| {
                let cookie_store = cookie_store_clone.read().unwrap();
                cookie_store.handle_request(request);
            })
        } else {
            self.inner.send(addr, port, req)
        }
    }

    pub fn recv<S: ApiResponse>(
        &mut self,
        key: &ResponseKey<S>,
    ) -> Option<Result<S, ResponseError>> {
        self.inner.recv(key)
    }

    pub fn cookie_header_value(&self) -> Option<String> {
        let cookie_store = self.cookie_store_opt.clone()?;
        let cookie_store = cookie_store.read().unwrap();
        cookie_store.cookie_header_value()
    }
}
