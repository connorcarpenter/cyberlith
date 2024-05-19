use std::time::Duration;

use crate::{headers::HeaderStore, Method};

/// A simple HTTP request.
#[derive(Clone)]
pub struct Request {
    /// "GET", "POST", …
    pub method: Method,

    /// https://…
    pub url: String,

    /// The data you send with e.g. "POST".
    pub body: Vec<u8>,

    /// ("Accept", "*/*"), …
    headers: HeaderStore,
}

impl Request {
    pub fn new(method: Method, url: &str, body: Vec<u8>) -> Self {
        let mut headers = HeaderStore::new();

        if (method == Method::Get || method == Method::Options) && !body.is_empty() {
            panic!("GET/OPTIONS requests cannot have a body");
        } else {
            headers.insert("Content-Length".to_string(), body.len().to_string());
        }

        Self {
            method,
            url: url.to_string(),
            body,
            headers,
        }
    }

    pub fn set_header_store(&mut self, header_store: HeaderStore) {
        self.headers = header_store;
    }

    pub fn has_header(&self, name: &str) -> bool {
        self.headers.has(name)
    }

    pub fn get_header_first(&self, name: &str) -> Option<&String> {
        self.headers.get(name)?.first()
    }

    pub fn get_headers(&self, name: &str) -> Option<&Vec<String>> {
        self.headers.get(name)
    }

    pub fn insert_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    pub fn remove_header_all(&mut self, name: &str) {
        self.headers.remove_all(name);
    }

    pub fn headers_iter(&self) -> std::collections::hash_map::Iter<'_, String, Vec<String>> {
        self.headers.iter()
    }
}

/// Request options
pub struct RequestOptions {
    pub timeout_opt: Option<Duration>,
}
