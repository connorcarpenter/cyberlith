use http_client_shared::fetch_blocking;
use std::sync::{Arc, RwLock};

use http_common::{Method, Request, Response};

use crate::http::CookieStore;

pub(crate) fn head_request(cookie_store: Arc<RwLock<CookieStore>>, url: &str) -> Response {
    let mut request = Request::new(Method::Head, url, Vec::new());

    {
        let cookie_store = cookie_store.read().unwrap();
        cookie_store.handle_request(&mut request);
    }

    let response = match fetch_blocking(&request, None) {
        Ok(response) => response,
        Err(e) => {
            panic!("Unhandled on purpose. Make sure Gateway & Content Server are running! Error received: {:?}", e.to_string())
        }
    };

    let mut cookie_store = cookie_store.write().unwrap();
    cookie_store.handle_response(&response);

    response
}
