use std::{collections::BTreeMap, time::Duration};

/// HTTP Method
#[derive(Clone, Eq, PartialEq)]
pub enum Method {
    Get,
    Post,
    Delete,
    Put,
    Head,
    Patch,
    Options,
}

impl Method {
    pub fn from_str(val: &str) -> Result<Self, ()> {
        match val {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "DELETE" => Ok(Self::Delete),
            "PUT" => Ok(Self::Put),
            "HEAD" => Ok(Self::Head),
            "PATCH" => Ok(Self::Patch),
            "OPTIONS" => Ok(Self::Options),
            _ => Err(()),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Delete => "DELETE",
            Self::Put => "PUT",
            Self::Head => "HEAD",
            Self::Patch => "PATCH",
            Self::Options => "OPTIONS",
        }
    }
}

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
    pub headers: BTreeMap<String, String>,
}

impl Request {
    pub fn new(method: Method, url: &str, body: Vec<u8>) -> Self {
        let mut headers = BTreeMap::new();

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
}

/// Request options
pub struct RequestOptions {
    pub timeout_opt: Option<Duration>,
}

/// Response from a completed HTTP request.
#[derive(Clone, Eq, PartialEq)]
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
    pub headers: BTreeMap<String, String>,

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
            headers: BTreeMap::new(),
            body: Vec::new(),
        }
    }
}

impl Response {
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
        let mut headers = BTreeMap::new();
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
        let mut headers = BTreeMap::new();

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
