
use std::{collections::HashMap, fs, io, fs::File, io::{Read, Write}};

use chrono::{DateTime, Utc, ParseError};

use http_common::{Request, Response};
use logging::{info, warn};

pub struct CookieStore {
    cookies: HashMap<String, (String, Option<DateTime<Utc>>)>,
    cookies_dir: String,
}

impl CookieStore {

    pub(crate) fn new() -> Self {
        Self::new_impl("cookies")
    }

    #[allow(dead_code)]
    pub(crate) fn test() -> Self {
        Self::new_impl("cookies_test")
    }

    fn new_impl(dir: &str) -> Self {
        let mut cookie_store = Self {
            cookies: HashMap::new(),
            cookies_dir: dir.to_string(),
        };
        cookie_store.load_cookies_from_files().unwrap();
        cookie_store.cleanup_expired_cookies().unwrap();
        cookie_store
    }

    fn load_cookies_from_files(&mut self) -> io::Result<()> {
        // check if the cookies directory exists, if not, create it
        if !fs::metadata(&self.cookies_dir).is_ok() {
            fs::create_dir_all(&self.cookies_dir)?;
        }
        for entry in fs::read_dir(&self.cookies_dir)? {
            let entry = entry?;
            let file_name = entry.file_name().into_string().unwrap();
            let cookie_name = file_name.trim_end_matches(".cookie");
            let mut file = File::open(entry.path())?;
            let mut cookie_value = String::new();
            file.read_to_string(&mut cookie_value)?;
            if let Some((value, expires)) = parse_cookie_value(&cookie_value) {
                self.cookies.insert(cookie_name.to_string(), (value, expires));
            }
        }
        Ok(())
    }

    pub(crate) fn handle_request(&self, request: &mut Request) {
        info!("Handling request: {:?}", request.url);
        if let Some(cookie_header_value) = self.cookie_header_value() {
            info!("Adding cookie header to request: {}", cookie_header_value);
            request.insert_header("Cookie", &cookie_header_value);
        }
    }

    pub fn cookie_header_value(&self) -> Option<String> {
        if self.cookies.is_empty() {
            return None;
        }
        let now = Utc::now();
        let cookie_header_value = self.cookies.iter()
            .filter_map(|(name, (value, expires))| {
                if let Some(expires) = expires {
                    if *expires > now {
                        Some(format!("{}={}", name, value))
                    } else {
                        None
                    }
                } else {
                    Some(format!("{}={}", name, value))
                }
            })
            .collect::<Vec<_>>()
            .join("; ");
        if cookie_header_value.is_empty() {
            None
        } else {
            Some(cookie_header_value)
        }
    }

    pub(crate) fn handle_response(&mut self, response: &Response) {
        info!("Handling response: {:?}", response.status);
        if let Some(set_cookie_headers) = response.get_headers("Set-Cookie") {
            let now = Utc::now();
            let mut new_cookies = HashMap::new();
            for header_value in set_cookie_headers {
                if let Some((name, value, expires)) = extract_cookie_from_header(header_value) {
                    if expires.map_or(true, |exp| exp > now) {
                        new_cookies.insert(name.clone(), (value.clone(), expires.clone()));
                    }
                }
            }
            for (name, (value, expires)) in new_cookies {
                info!("Adding cookie to store: name={}, value={}, expires={:?}", name, value, expires);
                self.cookies.insert(name.clone(), (value.clone(), expires.clone()));
                self.write_cookie_to_file(&name, &value, expires).unwrap();
            }
        }
    }

    fn write_cookie_to_file(&self, name: &str, value: &str, expires: Option<DateTime<Utc>>) -> io::Result<()> {
        info!("Writing cookie to file: name={}, value={}, expires={:?}", name, value, expires);
        let file_path = format!("{}/{}.cookie", &self.cookies_dir, name);
        let mut file = File::create(file_path)?;
        file.write_all(value.as_bytes())?;
        if let Some(expires) = expires {
            file.write_all(b"\n")?;
            file.write_all(expires.to_rfc3339().as_bytes())?;
        }
        Ok(())
    }

    fn cleanup_expired_cookies(&mut self) -> io::Result<()> {
        let mut expired_cookies = Vec::new();
        for (name, (_, expires)) in &self.cookies {
            if let Some(expires) = expires {
                if *expires <= Utc::now() {
                    expired_cookies.push(name.clone());
                }
            }
        }
        for name in expired_cookies {
            self.cookies.remove(&name);
            let file_path = format!("{}/{}.cookie", &self.cookies_dir, name);
            fs::remove_file(file_path)?;
        }
        Ok(())
    }
}

fn extract_cookie_from_header(header_value: &str) -> Option<(String, String, Option<DateTime<Utc>>)> {
    let mut cookies: Vec<(&str, &str)> = header_value
        .split(';')
        .map(str::trim)
        .filter_map(|part| part.split_once('='))
        .map(|(name, value)| (name.trim(), value.trim()))
        .collect();

    if cookies.len() > 0 {
        let (name, value) = cookies.remove(0);
        if name.to_lowercase() == "expires" {
            return None;
        }
        if value.is_empty() {
            return None;
        }
        let expires = cookies
            .iter()
            .find(|&&(attr, _)| attr.trim().to_lowercase() == "expires")
            .and_then(|&(_, val)| parse_cookie_expires(val).ok());

        Some((name.to_string(), value.to_string(), expires))
    } else {
        None
    }
}

fn parse_cookie_expires(value: &str) -> Result<DateTime<Utc>, ParseError> {
    let value = value.trim();
    info!("Parsing cookie expires: {}", value);

    let output = DateTime::parse_from_rfc2822(value).map(|dt| dt.with_timezone(&Utc));
    if let Err(e) = &output {
        warn!("error: {:?}", e);
    }
    output
}

fn parse_cookie_value(value: &str) -> Option<(String, Option<DateTime<Utc>>)> {
    let mut parts = value.split('\n');
    if let Some(cookie_value) = parts.next() {
        let expires = parts.next().and_then(|expires| parse_cookie_expires(expires).ok());
        Some((cookie_value.to_string(), expires))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use http_common::Method;

    const TEST_COOKIES_DIR: &str = "cookies_test";

    fn cleanup_test_cookies_dir() {
        println!("Cleaning up test cookies directory");
        if let Ok(entries) = fs::read_dir(TEST_COOKIES_DIR) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Err(err) = fs::remove_file(entry.path()) {
                        eprintln!("Failed to remove file: {}", err);
                    }
                }
            }
        }
    }

    #[test]
    fn test_extract_cookie_from_header_single_cookie() {
        let header = "test=value";
        let result = extract_cookie_from_header(header);
        assert_eq!(result, Some(("test".to_string(), "value".to_string(), None)));
    }

    #[test]
    fn test_extract_cookie_from_header_with_expires() {
        let header = "test=value; Expires=Sun, 24 Apr 2022 11:34:32 GMT";
        let (name, value, expires) = extract_cookie_from_header(header).unwrap();
        assert_eq!(name, "test".to_string());
        assert_eq!(value, "value".to_string());
        assert_eq!(expires.is_some(), true);
    }

    #[test]
    fn test_extract_cookie_from_header_multiple_attributes() {
        logging::initialize();
        let header = "test=value; Path=/; Expires=Sun, 24 Apr 2022 11:34:32 GMT; Secure";
        let (name, value, expires) = extract_cookie_from_header(header).unwrap();
        assert_eq!(name, "test".to_string());
        assert_eq!(value, "value".to_string());
        assert_eq!(expires.is_some(), true);
    }

    #[test]
    fn test_extract_cookie_from_header_missing_value() {
        let header = "test=";
        let result = extract_cookie_from_header(header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_cookie_from_header_missing_expires() {
        let header = "test=value; Path=/";
        let (name, value, expires) = extract_cookie_from_header(header).unwrap();
        assert_eq!(name, "test".to_string());
        assert_eq!(value, "value".to_string());
        assert_eq!(expires, None);
    }

    #[test]
    fn test_extract_cookie_from_header_no_cookie() {
        let header = "Expires=Sun, 24 Apr 2022 11:34:32 GMT";
        let result = extract_cookie_from_header(header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_handle_request_no_cookies() {
        logging::initialize();
        cleanup_test_cookies_dir(); // Clean up test_cookies directory before running the test
        let mut request = Request::new(Method::Get, "", Vec::new());
        let cookie_store = CookieStore::test();
        cookie_store.handle_request(&mut request);
        assert_eq!(request.get_headers("Cookie"), None);
    }

    #[test]
    fn test_handle_request_with_cookies() {
        logging::initialize();
        cleanup_test_cookies_dir(); // Clean up test_cookies directory before running the test
        let mut request = Request::new(Method::Get, "", Vec::new());
        let mut cookie_store = CookieStore::test();
        cookie_store.cookies.insert("test".to_string(), ("value".to_string(), None));
        cookie_store.handle_request(&mut request);
        assert_eq!(request.get_headers("Cookie"), Some(&vec!["test=value".to_string()]));
    }

    #[test]
    fn test_handle_response_add_cookie() {
        logging::initialize();
        cleanup_test_cookies_dir(); // Clean up test_cookies directory before running the test
        let mut response = Response::default();
        response.insert_header("Set-Cookie", "test=value; Expires=Sun, 24 Apr 2026 11:34:32 GMT");
        let mut cookie_store = CookieStore::test();
        cookie_store.handle_response(&response);
        assert_eq!(cookie_store.cookies.len(), 1);
        assert_eq!(cookie_store.cookies.contains_key("test"), true);
        let (value, _) = cookie_store.cookies.get("test").unwrap();
        assert_eq!(value, "value");
    }

    #[test]
    fn test_handle_response_ignore_expired_cookie() {
        logging::initialize();
        cleanup_test_cookies_dir(); // Clean up test_cookies directory before running the test
        let mut response = Response::default();
        response.insert_header("Set-Cookie", "expired_cookie=value; Expires=Sat, 24 Apr 2021 11:34:32 GMT");
        let mut cookie_store = CookieStore::test();
        cookie_store.handle_response(&response);
        assert_eq!(cookie_store.cookies.len(), 0);
    }

    #[test]
    fn test_handle_response_ignore_expired_cookie_with_existing_cookies() {
        logging::initialize();
        cleanup_test_cookies_dir(); // Clean up test_cookies directory before running the test
        let mut response = Response::default();
        response.insert_header("Set-Cookie", "expired_cookie=value; Expires=Sat, 24 Apr 2021 11:34:32 GMT");
        let mut cookie_store = CookieStore::test();
        cookie_store.cookies.insert("existing_cookie".to_string(), ("existing_value".to_string(), None));
        cookie_store.handle_response(&response);
        assert_eq!(cookie_store.cookies.len(), 1);
        assert_eq!(cookie_store.cookies.contains_key("existing_cookie"), true);
    }
}