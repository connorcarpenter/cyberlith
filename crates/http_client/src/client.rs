
use crate::{types::{Request, Response}, convert::{request_http_to_ehttp, response_ehttp_to_http}};

pub struct HttpClient;

impl HttpClient {
    pub async fn send(request: Request) -> Result<Response, ()> {
        let ehttp_req = request_http_to_ehttp(request)?;
        let ehttp_res = ehttp::fetch_async(ehttp_req).await.map_err(|_| ())?;
        let http_res = response_ehttp_to_http(ehttp_res)?;
        Ok(http_res)
    }
}