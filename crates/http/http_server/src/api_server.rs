use std::net::SocketAddr;

use smol::future::Future;

use http_common::{ApiRequest, ApiResponse, Method, Request, Response, ResponseError};
use logging::info;

use crate::{
    endpoint::{EndpointRef, EndpointFunc, Endpoint},
    log_util, Server,
};

// serves API endpoint with typed requests & responses
pub trait ApiServer {
    fn api_endpoint<
        TypeRequest: 'static + ApiRequest,
        TypeResponse: 'static + Send + Sync + Future<Output = Result<TypeRequest::Response, ResponseError>>,
        Handler: 'static + Send + Sync + Fn(SocketAddr, TypeRequest) -> TypeResponse,
    >(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
        handler: Handler,
    ) -> EndpointRef;

    fn raw_endpoint<
        ResponseType: 'static + Send + Sync + Future<Output = Result<Response, ResponseError>>,
        Handler: 'static + Send + Sync + Fn(SocketAddr, Request) -> ResponseType,
    >(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
        allow_origin_opt: Option<&str>,
        incoming_method: Method,
        incoming_path: &str,
        handler: Handler,
    ) -> EndpointRef;
}

impl ApiServer for Server {
    fn api_endpoint<
        TypeRequest: 'static + ApiRequest,
        TypeResponse: 'static + Send + Sync + Future<Output = Result<TypeRequest::Response, ResponseError>>,
        Handler: 'static + Send + Sync + Fn(SocketAddr, TypeRequest) -> TypeResponse,
    >(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
        handler: Handler,
    ) -> EndpointRef {
        let method = TypeRequest::method();
        let path = TypeRequest::path();

        let endpoint_path = format!("{} /{}", method.as_str(), path);

        info!("endpoint: {}", endpoint_path);
        let endpoint_func =
            get_endpoint_func::<TypeRequest, TypeResponse, Handler>(host_name, handler);
        let incoming_host =
            incoming_host.map(|(rq, rdopt)| (rq.to_string(), rdopt.map(|rd| rd.to_string())));
        let new_endpoint = Endpoint::new(endpoint_func, incoming_host);
        self.internal_insert_endpoint(endpoint_path.clone(), new_endpoint);
        EndpointRef::new(self, endpoint_path)
    }

    fn raw_endpoint<
        ResponseType: 'static + Send + Sync + Future<Output = Result<Response, ResponseError>>,
        Handler: 'static + Send + Sync + Fn(SocketAddr, Request) -> ResponseType,
    >(
        &mut self,
        host_name: &str,
        incoming_host: Option<(&str, Option<&str>)>,
        allow_origin_opt: Option<&str>,
        incoming_method: Method,
        incoming_path: &str,
        handler: Handler,
    ) -> EndpointRef {
        let endpoint_path = format!("{} /{}", incoming_method.as_str(), incoming_path);

        info!("endpoint: {}", endpoint_path);
        let endpoint_func =
            get_endpoint_raw_func::<ResponseType, Handler>(host_name, allow_origin_opt, handler);
        let incoming_host =
            incoming_host.map(|(rq, rdopt)| (rq.to_string(), rdopt.map(|rd| rd.to_string())));
        let new_endpoint = Endpoint::new(endpoint_func, incoming_host);
        self.internal_insert_endpoint(endpoint_path.clone(), new_endpoint);
        EndpointRef::new(self, endpoint_path)
    }
}

fn get_endpoint_func<
    TypeRequest: 'static + ApiRequest,
    TypeResponse: 'static + Send + Sync + Future<Output = Result<TypeRequest::Response, ResponseError>>,
    Handler: 'static + Send + Sync + Fn(SocketAddr, TypeRequest) -> TypeResponse,
>(
    host_name: &str,
    handler: Handler,
) -> EndpointFunc {
    let host_name = host_name.to_string();
    Box::new(move |addr: SocketAddr, pure_request: Request| {
        let host_name = host_name.clone();
        let incoming_method = pure_request.method.clone();
        let incoming_path = pure_request.url.clone();

        let Ok(typed_request) = TypeRequest::from_request(pure_request) else {
            // serde error!
            return Box::pin(async move { Err(ResponseError::SerdeError) });
        };

        // success!

        let typed_future = handler(addr, typed_request);

        // convert typed future to pure future
        let pure_future = async move {
            let host_name = host_name.clone();
            let logged_host_url = format!("{} {}", incoming_method.as_str(), incoming_path);

            logging::info!("[");
            log_util::recv_req(&host_name, &logged_host_url, TypeRequest::name());

            let typed_response = typed_future.await;

            let response_name;

            let outgoing_response = match typed_response {
                Ok(typed_response) => {
                    response_name = TypeRequest::Response::name().to_string();
                    let pure_response = typed_response.to_response();
                    Ok(pure_response)
                }
                Err(err) => {
                    response_name = format!("Error: {}", err.to_string().as_str());
                    Err(err)
                },
            };

            log_util::send_res(&host_name, response_name.as_str());
            logging::info!("]");

            return outgoing_response;
        };

        return Box::pin(pure_future);
    })
}

fn get_endpoint_raw_func<
    ResponseType: 'static + Send + Sync + Future<Output = Result<Response, ResponseError>>,
    Handler: 'static + Send + Sync + Fn(SocketAddr, Request) -> ResponseType,
>(
    host_name: &str,
    allow_origin_opt: Option<&str>,
    handler: Handler,
) -> EndpointFunc {
    let host_name = host_name.to_string();
    let allow_origin_opt = allow_origin_opt.map(|s| s.to_string());
    Box::new(move |addr: SocketAddr, pure_request: Request| {
        let host_name = host_name.clone();
        let allow_origin_opt = allow_origin_opt.clone();
        let incoming_method = pure_request.method.clone();
        let incoming_path = pure_request.url.clone();
        let mut response_name = "error".to_string();

        let handler_func = handler(addr, pure_request);

        let pure_future = async move {
            let host_name = host_name.clone();
            let allow_origin_opt = allow_origin_opt.clone();
            let logged_host_url = format!("{} {}", incoming_method.as_str(), incoming_path);

            logging::info!("[");
            log_util::recv_req(&host_name, &logged_host_url, "");

            let mut response_result = handler_func.await;
            if let Ok(response) = response_result.as_mut() {

                response_name = format!("{} {}", response.status, response.status_text);

                if let Some(allow_origin) = allow_origin_opt {
                    while response.has_header("access-control-allow-origin") {
                        response.remove_header("access-control-allow-origin");
                    }
                    response.set_header("access-control-allow-origin", &allow_origin);
                }
            }

            log_util::send_res(&host_name, &response_name);
            logging::info!("]");

            response_result
        };

        return Box::pin(pure_future);
    })
}
