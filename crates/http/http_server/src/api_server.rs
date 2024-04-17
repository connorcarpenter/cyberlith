use std::{pin::Pin, net::SocketAddr};

use smol::future::Future;

use http_common::{ApiRequest, ApiResponse, Request, Response, ResponseError};
use logging::info;

use crate::Server;

// serves API endpoint with typed requests & responses
pub trait ApiServer {
    fn endpoint<
        TypeRequest: 'static + ApiRequest,
        TypeResponse: 'static + Send + Sync + Future<Output = Result<TypeRequest::Response, ResponseError>>,
        Handler: 'static + Send + Sync + FnMut((SocketAddr, TypeRequest)) -> TypeResponse,
    >(
        &mut self,
        handler: Handler,
    );
}

impl ApiServer for Server {
    fn endpoint<
        TypeRequest: 'static + ApiRequest,
        TypeResponse: 'static + Send + Sync + Future<Output = Result<TypeRequest::Response, ResponseError>>,
        Handler: 'static + Send + Sync + FnMut((SocketAddr, TypeRequest)) -> TypeResponse,
    >(
        &mut self,
        handler: Handler,
    ) {
        let method = TypeRequest::method();
        let path = TypeRequest::path();

        let endpoint_path = format!("{} /{}", method.as_str(), path);

        info!("endpoint: {}", endpoint_path);
        let new_endpoint = endpoint_2::<TypeRequest, TypeResponse, Handler>(handler);
        self.internal_insert_endpoint(endpoint_path, new_endpoint);
    }
}

fn endpoint_2<
    TypeRequest: 'static + ApiRequest,
    TypeResponse: 'static + Send + Sync + Future<Output = Result<TypeRequest::Response, ResponseError>>,
    Handler: 'static + Send + Sync + FnMut((SocketAddr, TypeRequest)) -> TypeResponse,
>(
    mut handler: Handler,
) -> Box<
    dyn 'static
    + Send
    + Sync
    + FnMut(
        (SocketAddr, Request),
    ) -> Pin<
        Box<dyn 'static + Send + Sync + Future<Output = Result<Response, ResponseError>>>,
    >,
> {
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