use std::collections::BTreeMap;
use std::str::FromStr;
use http::{HeaderName, StatusCode};

pub(crate) fn request_ehttp_to_http(ehttp_req: ehttp::Request) -> Result<http::Request<String>, ()> {

    // body
    let Ok(ehttp_body) = String::from_utf8(ehttp_req.body) else {
        return Err(());
    };
    let mut http_req = http::Request::new(ehttp_body);

    // method
    *http_req.method_mut() = method_ehttp_to_http(ehttp_req.method)?;

    // url
    let Ok(http_uri) = ehttp_req.url.parse() else {
        return Err(());
    };
    *http_req.uri_mut() = http_uri;

    // headers
    for (header_name, header_value) in ehttp_req.headers {
        let header_name = HeaderName::from_str(&header_name).map_err(|_| ())?;
        let header_value = http::HeaderValue::from_str(header_value.as_str()).map_err(|_| ())?;
        http_req.headers_mut().insert(header_name, header_value);
    }

    Ok(http_req)
}

pub(crate) fn request_http_to_ehttp(http_req: http::Request<String>) -> Result<ehttp::Request, ()> {

        // body
        let ehttp_body = http_req.body().as_bytes().to_owned();

        // method
        let ehttp_method = method_http_to_ehttp(http_req.method())?;

        // url
        let ehttp_url = http_req.uri().to_string();

        // headers
        let mut ehttp_headers = BTreeMap::new();
        for (header_name, header_value) in http_req.headers() {
            let header_value = header_value.to_str().map_err(|_| ())?;
            ehttp_headers.insert(header_name.to_string(), header_value.to_owned());
        }

        Ok(ehttp::Request {
            body: ehttp_body,
            method: ehttp_method,
            url: ehttp_url,
            headers: ehttp_headers,
        })
}

pub(crate) fn method_ehttp_to_http(ehttp_method: String) -> Result<http::Method, ()> {
    match ehttp_method.as_str() {
        "GET" => Ok(http::Method::GET),
        "POST" => Ok(http::Method::POST),
        "PUT" => Ok(http::Method::PUT),
        "DELETE" => Ok(http::Method::DELETE),
        _ => Err(()),
    }
}

pub(crate) fn method_http_to_ehttp(http_method: &http::Method) -> Result<String, ()> {
    match *http_method {
        http::Method::GET => Ok("GET".to_owned()),
        http::Method::POST => Ok("POST".to_owned()),
        http::Method::PUT => Ok("PUT".to_owned()),
        http::Method::DELETE => Ok("DELETE".to_owned()),
        _ => Err(()),
    }
}

pub(crate) fn response_ehttp_to_http(ehttp_res: ehttp::Response) -> Result<http::Response<String>, ()> {

    // body
    let Ok(http_body) = String::from_utf8(ehttp_res.bytes) else {
        return Err(());
    };
    let mut http_res = http::Response::new(http_body);

    // status
    *http_res.status_mut() = StatusCode::from_u16(ehttp_res.status).map_err(|_| ())?;

    // headers
    for (header_name, header_value) in ehttp_res.headers {
        let header_name = HeaderName::from_str(&header_name).map_err(|_| ())?;
        let header_value = http::HeaderValue::from_str(header_value.as_str()).map_err(|_| ())?;
        http_res.headers_mut().insert(header_name, header_value);
    }

    Ok(http_res)
}