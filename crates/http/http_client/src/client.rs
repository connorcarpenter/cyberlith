use http_common::{ApiRequest, ApiResponse, RequestOptions, ResponseError};

pub struct HttpClient;

impl HttpClient {
    pub async fn send<Q: ApiRequest>(
        addr: &str,
        port: u16,
        api_req: Q,
    ) -> Result<Q::Response, ResponseError> {
        let http_req = api_req.to_request(addr, port);
        let http_res = http_client_shared::fetch_async(http_req).await?;
        let http_res = http_res.to_result()?;
        let http_res = Q::Response::from_response(http_res)?;
        return Ok(http_res);
    }

    pub async fn send_with_options<Q: ApiRequest>(
        addr: &str,
        port: u16,
        api_req: Q,
        request_options: RequestOptions,
    ) -> Result<Q::Response, ResponseError> {
        let http_req = api_req.to_request(addr, port);
        let http_res = http_client_shared::fetch_async_with_options(http_req, request_options).await?;
        let http_res = http_res.to_result()?;
        let http_res = Q::Response::from_response(http_res)?;
        return Ok(http_res);
    }
}
