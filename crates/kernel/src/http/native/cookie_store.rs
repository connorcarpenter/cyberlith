
use std::{collections::HashMap, fs, io, fs::File, io::{Read, Write}};
use std::num::ParseIntError;

use chrono::{DateTime, ParseError, Utc};

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
            if let Some((value, expires)) = parse_file_cookie_value(&cookie_value) {
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
        // info!("handling Response: status: {:?}", response.status);

        // for (name, values) in response.headers_iter() {
        //     info!("Response Header: `{}` [", name);
        //     for value in values {
        //         info!("{}", value);
        //     }
        //     info!("]")
        // }

        if let Some(set_cookie_headers) = response.get_headers("Set-Cookie") {
            let now = Utc::now();
            let mut new_cookies = HashMap::new();
            let mut removed_cookies = Vec::new();
            for header_value in set_cookie_headers {
                if let Some((name, value, max_age)) = extract_cookie_from_header(header_value) {
                    if max_age.map_or(true, |exp| exp > 0) {
                        new_cookies.insert(name.clone(), (value.clone(), max_age.clone()));
                    } else {
                        removed_cookies.push(name.clone());
                    }
                }
            }
            for (name, (value, max_age_opt)) in new_cookies {
                // info!("Adding cookie to store: name={}, value={}, max-age={:?}", name, value, max_age_opt);
                let expires = max_age_opt.map(|max_age| now + chrono::Duration::seconds(max_age as i64));
                self.cookies.insert(name.clone(), (value.clone(), expires.clone()));
                self.write_cookie_to_file(&name, &value, expires).unwrap();
            }
            for name in removed_cookies {
                // info!("Removing cookie from store: name={}", name);
                self.cookies.remove(&name);
                let file_path = format!("{}/{}.cookie", &self.cookies_dir, name);
                let _ = fs::remove_file(file_path);
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

fn extract_cookie_from_header(header_value: &str) -> Option<(String, String, Option<u32>)> {
    let mut cookies: Vec<(&str, &str)> = header_value
        .split(';')
        .map(str::trim)
        .filter_map(|part| part.split_once('='))
        .map(|(name, value)| (name.trim(), value.trim()))
        .collect();

    if cookies.len() > 0 {
        let (name, value) = cookies.remove(0);
        if name.to_lowercase() == "max-age" {
            return None;
        }
        if value.is_empty() {
            return None;
        }
        let max_age = cookies
            .iter()
            .find(|&&(attr, _)| attr.trim().to_lowercase() == "max-age")
            .and_then(|&(_, val)| parse_cookie_max_age(val).ok());

        Some((name.to_string(), value.to_string(), max_age))
    } else {
        None
    }
}

fn parse_cookie_expires(value: &str) -> Result<DateTime<Utc>, ParseError> {
    let value = value.trim();
    info!("Parsing cookie expires: {}", value);

    let output = DateTime::parse_from_rfc3339(value).map(|dt| dt.with_timezone(&Utc));
    if let Err(e) = &output {
        warn!("error: {:?}", e);
    }
    output
}

fn parse_cookie_max_age(value: &str) -> Result<u32, ParseIntError> {
    let value = value.trim();
    info!("Parsing cookie max-age: {}", value);
    value.parse::<u32>()
}

fn parse_file_cookie_value(value: &str) -> Option<(String, Option<DateTime<Utc>>)> {
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
    fn test_extract_cookie_from_header_with_max_age() {
        let header = "test=value; Max-Age=3600";
        let (name, value, max_age) = extract_cookie_from_header(header).unwrap();
        assert_eq!(name, "test".to_string());
        assert_eq!(value, "value".to_string());
        assert_eq!(max_age.is_some(), true);
        assert_eq!(max_age.unwrap(), 3600);
    }

    #[test]
    fn test_extract_cookie_from_header_multiple_attributes() {
        logging::initialize();
        let header = "test=value; Secure; Path=/; Max-Age=3600";
        let (name, value, max_age) = extract_cookie_from_header(header).unwrap();
        assert_eq!(name, "test".to_string());
        assert_eq!(value, "value".to_string());
        assert_eq!(max_age.is_some(), true);
        assert_eq!(max_age.unwrap(), 3600);
    }

    #[test]
    fn test_extract_cookie_from_header_missing_value() {
        let header = "test=";
        let result = extract_cookie_from_header(header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_cookie_from_header_missing_max_age() {
        let header = "test=value; Path=/";
        let (name, value, expires) = extract_cookie_from_header(header).unwrap();
        assert_eq!(name, "test".to_string());
        assert_eq!(value, "value".to_string());
        assert_eq!(expires, None);
    }

    #[test]
    fn test_extract_cookie_from_header_no_cookie() {
        let header = "Max-Age=3600";
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
        response.insert_header("Set-Cookie", "test=value; Max-Age=3600");
        let mut cookie_store = CookieStore::test();
        cookie_store.handle_response(&response);
        assert_eq!(cookie_store.cookies.len(), 1);
        assert_eq!(cookie_store.cookies.contains_key("test"), true);
        let (value, _) = cookie_store.cookies.get("test").unwrap();
        assert_eq!(value, "value");
    }

    #[test]
    fn test_handle_response_add_two_cookies() {
        logging::initialize();
        cleanup_test_cookies_dir(); // Clean up test_cookies directory before running the test
        let mut response = Response::default();
        response.insert_header("Set-Cookie", "test1=value1; Max-Age=3600");
        response.insert_header("Set-Cookie", "test2=value2; Max-Age=3600");
        let mut cookie_store = CookieStore::test();
        cookie_store.handle_response(&response);
        assert_eq!(cookie_store.cookies.len(), 2);
        assert_eq!(cookie_store.cookies.contains_key("test1"), true);
        let (value, _) = cookie_store.cookies.get("test1").unwrap();
        assert_eq!(value, "value1");
        assert_eq!(cookie_store.cookies.contains_key("test2"), true);
        let (value, _) = cookie_store.cookies.get("test2").unwrap();
        assert_eq!(value, "value2");
    }

    #[test]
    fn test_handle_response_ignore_expired_cookie() {
        logging::initialize();
        cleanup_test_cookies_dir(); // Clean up test_cookies directory before running the test
        let mut response = Response::default();
        response.insert_header("Set-Cookie", "expired_cookie=value; Max-Age=0");
        let mut cookie_store = CookieStore::test();
        cookie_store.handle_response(&response);
        assert_eq!(cookie_store.cookies.len(), 0);
    }

    #[test]
    fn test_handle_response_remove_expired_cookie() {
        logging::initialize();
        cleanup_test_cookies_dir(); // Clean up test_cookies directory before running the test
        let mut response = Response::default();
        response.insert_header("Set-Cookie", "expired_cookie=value; Max-Age=0");
        let mut cookie_store = CookieStore::test();
        cookie_store.cookies.insert("expired_cookie".to_string(), ("existing_value".to_string(), None));
        cookie_store.handle_response(&response);
        assert_eq!(cookie_store.cookies.len(), 0);
    }

    #[test]
    fn test_handle_response_remove_expired_cookie_with_max_age() {
        logging::initialize();
        cleanup_test_cookies_dir(); // Clean up test_cookies directory before running the test
        let mut response = Response::default();
        response.insert_header("Set-Cookie", "expired_cookie=value; Max-Age=0");
        let mut cookie_store = CookieStore::test();
        let expires = Utc::now() + chrono::Duration::seconds(3600);
        cookie_store.cookies.insert("expired_cookie".to_string(), ("existing_value".to_string(), Some(expires)));
        cookie_store.handle_response(&response);
        assert_eq!(cookie_store.cookies.len(), 0);
    }
}