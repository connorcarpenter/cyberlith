use std::net::SocketAddr;

use http_common::{ApiRequest, ApiResponse, ResponseError};

pub struct HttpClient;

impl HttpClient {
    pub async fn send<Q: ApiRequest>(addr: &SocketAddr, api_req: Q) -> Result<Q::Response, ResponseError> {
        let http_req = api_req.to_request(addr);
        let http_res = http_client_shared::fetch_async(http_req).await;
        match http_res {
            Ok(http_res_0) => {
                let api_res = Q::Response::from_response(http_res_0)?;
                Ok(api_res)
            }
            Err(err) => {
                return Err(ResponseError::EhttpError(err));
            }
        }
    }
}
