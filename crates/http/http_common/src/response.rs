use std::collections::hash_map::Iter;
use crate::{headers::HeaderStore, ResponseError};

/// Response from a completed HTTP request.
#[derive(Clone)]
pub struct Response {
    /// The URL we ended up at. This can differ from the request url when we have followed redirects.
    pub url: String,

    /// Did we get a 2xx response code?
    pub ok: bool,

    /// Status code (e.g. `404` for "File not found").
    pub status: u16,

    /// Status text (e.g. "File not found" for status code `404`).
    pub status_text: String,

    /// The returned headers. All header names are lower-case.
    headers: HeaderStore,

    /// The raw bytes of the response body.
    pub body: Vec<u8>,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            url: "".to_string(),
            ok: true,
            status: 200,
            status_text: "".to_string(),
            headers: HeaderStore::new(),
            body: Vec::new(),
        }
    }
}

impl Response {

    // headers
    pub fn has_header(&self, name: &str) -> bool {
        self.headers.has(name)
    }

    pub fn get_header_first(&self, name: &str) -> Option<&String> {
        self.headers.get(name)?.first()
    }

    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    pub fn remove_header_all(&mut self, name: &str) {
        self.headers.remove_all(name);
    }

    pub fn headers_iter(&self) -> Iter<'_, String, Vec<String>> {
        self.headers.iter()
    }

    pub fn to_result(self) -> Result<Self, ResponseError> {
        if self.ok {
            Ok(self)
        } else {
            Err(ResponseError::from_response(&self))
        }
    }

    /// Constructs a new Response indicating a successful request. // 200
    pub fn ok(old_url: &str) -> Self {
        Self {
            url: old_url.to_string(),
            ok: true,
            status: 200,
            status_text: "OK".to_string(),
            ..Default::default()
        }
    }

    /// Constructs a new Response indicating a redirect to a specific URL. // 302
    pub fn redirect(old_url: &str, new_url: &str) -> Self {
        // Create headers for redirect
        let mut headers = HeaderStore::new();
        headers.insert("Location".to_string(), new_url.to_string());

        Self {
            url: old_url.to_string(),
            ok: false,
            status: 302, // HTTP status code for redirection
            status_text: "Found".to_string(),
            headers,
            ..Default::default()
        }
    }

    /// Constructs a new Response indicating a not modified response. // 304
    pub fn not_modified(old_url: &str) -> Self {
        Self {
            url: old_url.to_string(),
            ok: false,
            status: 304,
            status_text: "Not Modified".to_string(),
            ..Default::default()
        }
    }

    /// Constructs a new Response indicating a not modified response. // 404
    pub fn not_found(old_url: &str) -> Self {
        Self {
            url: old_url.to_string(),
            ok: false,
            status: 404,
            status_text: "Not Found".to_string(),
            ..Default::default()
        }
    }

    /// Constructs a new Response indicating a too many requests response. // 429
    pub fn too_many_requests(old_url: &str, retry_after_secs: usize) -> Self {
        let mut headers = HeaderStore::new();

        headers.insert("Retry-After".to_string(), retry_after_secs.to_string());

        Self {
            url: old_url.to_string(),
            ok: false,
            status: 429,
            status_text: "Too Many Requests".to_string(),
            headers,
            ..Default::default()
        }
    }
}