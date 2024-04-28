use std::any::Any;
use std::net::SocketAddr;

use naia_serde::{BitReader, BitWriter};

use config::{
    REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR, SESSION_SERVER_RECV_ADDR,
    SESSION_SERVER_SIGNAL_PORT,
};
use http_client::{HttpClient, ResponseError};
use http_server::{Method, Request, Response};
use logging::{info, warn};

use region_server_http_proto::SessionConnectRequest;
use session_server_naia_proto::{
    messages::{FakeEntityConverter, Message, Auth as SessionAuth},
    protocol,
};

use crate::access_token_checker::middleware_impl;

pub(crate) async fn handler(
    _addr: SocketAddr,
    incoming_request: Request,
) -> Result<Response, ResponseError> {

    let host_name = "gateway";

    // call to region server with login request
    let connect_response = {
        let region_server = "region_server";
        let remote_addr = REGION_SERVER_RECV_ADDR;
        let remote_port = REGION_SERVER_PORT;
        let remote_method = Method::Post;
        let remote_path = "session/connect";

        let logged_remote_url = format!(
            "{} host:{}/{}",
            remote_method.as_str(),
            remote_port,
            remote_path
        );
        http_server::http_log_util::send_req(host_name, region_server, &logged_remote_url);

        let Ok(connect_response) =
            HttpClient::send(&remote_addr, remote_port, SessionConnectRequest).await
        else {
            warn!("Failed session_connect request to region server");
            return Err(ResponseError::InternalServerError(
                "failed session_connect to region server".to_string(),
            ));
        };

        http_server::http_log_util::recv_res(host_name, region_server, &logged_remote_url);

        connect_response
    };

    // forward original request to session server
    {
        let session_server = "session_server";
        let remote_addr = SESSION_SERVER_RECV_ADDR;
        let remote_port = SESSION_SERVER_SIGNAL_PORT.to_string();
        let remote_method = Method::Post;
        let remote_path = "session_rtc";

        let logged_remote_url = format!(
            "{} host:{}/{}",
            remote_method.as_str(),
            remote_port,
            remote_path
        );
        http_server::http_log_util::send_req(host_name, session_server, &logged_remote_url);

        let session_auth_bytes = {
            let session_auth = connect_response.session_auth.to_outer();

            // TODO: this operation is VERY heavy! We should cache the result
            let message_kinds = protocol().into().message_kinds;

            let mut writer = BitWriter::new();
            session_auth.write(&message_kinds, &mut writer, &mut FakeEntityConverter);
            let bytes = writer.to_bytes();

            // base64 encode
            base64::encode(&bytes)
        };

        let mut session_rtc_request = incoming_request.clone();
        session_rtc_request.url = format!("http://{}:{}/{}", remote_addr, remote_port, remote_path);
        session_rtc_request.set_header("Authorization", &session_auth_bytes);
        match http_client::raw::fetch_async(session_rtc_request).await {
            Ok(session_rtc_response) => {
                http_server::http_log_util::recv_res(host_name, session_server, &logged_remote_url);
                return Ok(session_rtc_response);
            }
            Err(err) => {
                warn!(
                    "Failed session_rtc request to session server: {}",
                    err.to_string()
                );
                return Err(ResponseError::InternalServerError(
                    "internal server error".to_string(),
                ));
            }
        }
    }
}

pub(crate) async fn auth_middleware(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> Option<Result<Response, ResponseError>> {

    let access_token: Option<String> = get_access_token_from_base64(&incoming_request);
    if access_token.is_some() {
        info!("found access_token in header: {}", access_token.as_ref().unwrap());
    } else {
        info!("no access_token found in header");
    }
    middleware_impl(incoming_addr, incoming_request, access_token).await
}

fn get_access_token_from_base64(incoming_request: &Request) -> Option<String> {
    let auth_header = incoming_request.get_header("authorization").map(|s| s.clone())?;
    let auth_bytes = base64::decode(&auth_header).ok()?;

    // TODO: this operation is VERY heavy! We should cache the result
    let message_kinds = protocol().into().message_kinds;
    let mut bit_reader = BitReader::new(&auth_bytes);
    let auth_message = message_kinds.read(&mut bit_reader, &FakeEntityConverter).ok()?;
    let auth_message_any = auth_message.clone().to_boxed_any();
    let auth_message: SessionAuth = Box::<dyn Any + 'static>::downcast::<SessionAuth>(auth_message_any)
        .ok()
        .map(|boxed_m| *boxed_m)
        .unwrap();
    let access_token = auth_message.token().to_string();

    Some(access_token)
}