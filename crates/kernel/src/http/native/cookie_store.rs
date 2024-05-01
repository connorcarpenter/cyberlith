use std::collections::HashMap;

use http_common::{Request, Response};

pub struct CookieStore {
    cookies: HashMap<String, String>,
}

impl CookieStore {
    pub(crate) fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    pub(crate) fn handle_request(&self, request: &mut Request) {
        if !self.cookies.is_empty() {
            let cookie_header_value = self.cookies.iter()
                .map(|(name, value)| format!("{}={}", name, value))
                .collect::<Vec<_>>()
                .join("; ");
            request.insert_header("Cookie", &cookie_header_value);
        }
    }

    pub(crate) fn handle_response(&mut self, response: &Response) {
        if let Some(set_cookie_headers) = response.get_headers("Set-Cookie") {
            let mut new_cookies = Vec::new();
            for header_value in set_cookie_headers {
                if let Some((name, value)) = extract_cookie_from_header(header_value) {
                    new_cookies.push((name, value));
                }
            }
            // Update existing cookies with the same name
            for (name, value) in new_cookies {
                self.cookies.insert(name, value);
            }
        }
    }
}

fn extract_cookie_from_header(header_value: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = header_value.split(';').collect();
    if let Some(cookie) = parts.first() {
        if let Some((name, value)) = cookie.split_once('=') {
            Some((name.trim().to_string(), value.trim().to_string()))
        } else {
            None
        }
    } else {
        None
    }
}