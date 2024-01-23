use http_common::{Method, Request, Response};

// pub(crate) fn request_ehttp_to_http(ehttp_req: ehttp::Request) -> Result<Request, ()> {
//     let mut http_req = Request::new(
//         method_ehttp_to_http(ehttp_req.method)?,
//         ehttp_req.url.as_str(),
//         ehttp_req.body,
//     );
//     http_req.headers = ehttp_req.headers;
//
//     Ok(http_req)
// }

pub(crate) fn request_http_to_ehttp(http_req: Request) -> Result<ehttp::Request, ()> {
    let ehttp_req = match http_req.method {
        Method::Get => {
            let ehttp_req = ehttp::Request::get(http_req.url.as_str());
            ehttp_req
        }
        Method::Post => {
            let Request {
                method: _,
                url: http_url,
                body: http_body,
                headers: http_headers,
            } = http_req;
            let body_length = http_body.len();
            let mut ehttp_req = ehttp::Request::post(http_url, http_body);
            for (key, value) in http_headers {
                ehttp_req.headers.insert(key, value);
            }
            ehttp_req.headers.insert("Content-Length".to_string(), format!("{}", body_length));
            ehttp_req
        }
        _ => panic!("unsupported!"),
    };
    Ok(ehttp_req)
}

// pub(crate) fn method_ehttp_to_http(ehttp_method: String) -> Result<Method, ()> {
//     match ehttp_method.as_str() {
//         "GET" => Ok(Method::Get),
//         "POST" => Ok(Method::Post),
//         "PUT" => Ok(Method::Put),
//         "DELETE" => Ok(Method::Delete),
//         _ => Err(()),
//     }
// }

// pub(crate) fn method_http_to_ehttp(http_method: &Method) -> Result<String, ()> {
//     match *http_method {
//         Method::Get => Ok("GET".to_owned()),
//         Method::Post => Ok("POST".to_owned()),
//         Method::Put => Ok("PUT".to_owned()),
//         Method::Delete => Ok("DELETE".to_owned()),
//         _ => Err(()),
//     }
// }

pub(crate) fn response_ehttp_to_http(ehttp_res: ehttp::Response) -> Result<Response, ()> {
    let mut http_res = Response::default();
    http_res.body = ehttp_res.bytes;
    http_res.url = ehttp_res.url;
    http_res.ok = ehttp_res.ok;
    http_res.status = ehttp_res.status;
    http_res.status_text = ehttp_res.status_text;
    http_res.headers = ehttp_res.headers;

    Ok(http_res)
}
