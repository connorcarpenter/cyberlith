use std::collections::HashMap;
use std::{fs, io};
use std::fs::File;
use std::io::{Read, Write};

use http_common::{Request, Response};

pub struct CookieStore {
    cookies: HashMap<String, String>,
}

impl CookieStore {

    const COOKIES_DIR: &'static str = "cookies";

    pub(crate) fn new() -> Self {
        let mut cookie_store = Self {
            cookies: HashMap::new(),
        };
        cookie_store.load_cookies_from_files().unwrap();
        cookie_store
    }

    fn load_cookies_from_files(&mut self) -> io::Result<()> {
        if !fs::metadata(Self::COOKIES_DIR)?.is_dir() {
            fs::create_dir_all(Self::COOKIES_DIR)?;
        }
        for entry in fs::read_dir(Self::COOKIES_DIR)? {
            let entry = entry?;
            let file_name = entry.file_name().into_string().unwrap();
            let cookie_name = file_name.trim_end_matches(".cookie");
            let mut file = File::open(entry.path())?;
            let mut cookie_value = String::new();
            file.read_to_string(&mut cookie_value)?;
            self.cookies.insert(cookie_name.to_string(), cookie_value);
        }
        Ok(())
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
                self.write_cookie_to_file(&name, &value).unwrap();
                self.cookies.insert(name, value);
            }
        }
    }

    fn write_cookie_to_file(&self, name: &str, value: &str) -> io::Result<()> {
        let file_path = format!("{}/{}.cookie", Self::COOKIES_DIR, name);
        let mut file = File::create(file_path)?;
        file.write_all(value.as_bytes())?;
        Ok(())
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