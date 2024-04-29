use std::collections::BTreeMap;

/// An HTTP response status line and headers used for the [`streaming`](crate::streaming) API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialResponse {
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
}
