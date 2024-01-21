use std::net::SocketAddr;

use http_common::{ApiRequest, ApiResponse};

use crate::convert::{request_http_to_ehttp, response_ehttp_to_http};

pub struct HttpClient;

impl HttpClient {
    pub async fn send<Q: ApiRequest>(addr: &SocketAddr, api_req: Q) -> Result<Q::Response, ()> {
        let http_req = api_req.to_request(addr);
        let ehttp_req = request_http_to_ehttp(http_req)?;
        let ehttp_res = ehttp::fetch_async(ehttp_req).await.map_err(|_| ())?;
        let http_res = response_ehttp_to_http(ehttp_res)?;
        let Ok(api_res) = Q::Response::from_response(http_res) else {
            return Err(());
        };
        return Ok(api_res);
    }
}
