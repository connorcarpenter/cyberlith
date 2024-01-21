use std::collections::BTreeMap;

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
        Self {
            method,
            url: url.to_string(),
            body,
            headers: BTreeMap::new(),
        }
    }
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
            body: vec![],
        }
    }
}