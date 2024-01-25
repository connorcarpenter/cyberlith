use std::net::SocketAddr;

use http_common::{ApiRequest, ApiResponse};

pub struct HttpClient;

impl HttpClient {
    pub async fn send<Q: ApiRequest>(addr: &SocketAddr, api_req: Q) -> Result<Q::Response, ()> {
        let http_req = api_req.to_request(addr);
        let http_res = http_client_shared::fetch_async(http_req).await.map_err(|_| ())?;
        let Ok(api_res) = Q::Response::from_response(http_res) else {
            return Err(());
        };
        return Ok(api_res);
    }
}
