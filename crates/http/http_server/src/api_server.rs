use std::net::SocketAddr;

use smol::future::Future;

use http_common::{ApiRequest, ApiResponse, Request, ResponseError};
use logging::info;

use crate::{Server, base_server::{Endpoint, EndpointFunc}};

// serves API endpoint with typed requests & responses
pub trait ApiServer {
    fn endpoint<
        TypeRequest: 'static + ApiRequest,
        TypeResponse: 'static + Send + Sync + Future<Output = Result<TypeRequest::Response, ResponseError>>,
        Handler: 'static + Send + Sync + Fn((SocketAddr, TypeRequest)) -> TypeResponse,
    >(
        &mut self,
        handler: Handler,
    );
}

impl ApiServer for Server {
    fn endpoint<
        TypeRequest: 'static + ApiRequest,
        TypeResponse: 'static + Send + Sync + Future<Output = Result<TypeRequest::Response, ResponseError>>,
        Handler: 'static + Send + Sync + Fn((SocketAddr, TypeRequest)) -> TypeResponse,
    >(
        &mut self,
        handler: Handler,
    ) {
        let method = TypeRequest::method();
        let path = TypeRequest::path();

        let endpoint_path = format!("{} /{}", method.as_str(), path);

        info!("endpoint: {}", endpoint_path);
        let endpoint_func = get_endpoint_func::<TypeRequest, TypeResponse, Handler>(handler);
        let new_endpoint = Endpoint::new(endpoint_func, None);
        self.internal_insert_endpoint(endpoint_path, new_endpoint);
    }
}

fn get_endpoint_func<
    TypeRequest: 'static + ApiRequest,
    TypeResponse: 'static + Send + Sync + Future<Output = Result<TypeRequest::Response, ResponseError>>,
    Handler: 'static + Send + Sync + Fn((SocketAddr, TypeRequest)) -> TypeResponse,
>(
    handler: Handler,
) -> EndpointFunc {
    Box::new(move |args: (SocketAddr, Request)| {
        let addr = args.0;
        let pure_request = args.1;

        let Ok(typed_request) = TypeRequest::from_request(pure_request) else {
            // serde error!
            return Box::pin(async move { Err(ResponseError::SerdeError) });
        };

        // success!

        let typed_future = handler((addr, typed_request));

        // convert typed future to pure future
        let pure_future = async move {
            let typed_response = typed_future.await;
            match typed_response {
                Ok(typed_response) => {
                    let pure_response = typed_response.to_response();
                    Ok(pure_response)
                }
                Err(err) => Err(err),
            }
        };

        return Box::pin(pure_future);
    })
}
