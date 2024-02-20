use http_common::{ApiRequest, ApiResponse, RequestOptions, ResponseError};

pub struct HttpClient;

impl HttpClient {
    pub async fn send<Q: ApiRequest>(
        addr: &str,
        port: u16,
        api_req: Q,
    ) -> Result<Q::Response, ResponseError> {
        let http_req = api_req.to_request(addr, port);
        let http_res = http_client_shared::fetch_async(http_req).await;
        match http_res {
            Ok(http_res_0) => {
                let api_res = Q::Response::from_response(http_res_0)?;
                Ok(api_res)
            }
            Err(err) => {
                return Err(err);
            }
        }
    }

    pub async fn send_with_options<Q: ApiRequest>(
        addr: &str,
        port: u16,
        api_req: Q,
        request_options: RequestOptions,
    ) -> Result<Q::Response, ResponseError> {
        let http_req = api_req.to_request(addr, port);
        let http_res =
            http_client_shared::fetch_async_with_options(http_req, request_options).await;
        match http_res {
            Ok(http_res_0) => {
                let api_res = Q::Response::from_response(http_res_0)?;
                Ok(api_res)
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
}
