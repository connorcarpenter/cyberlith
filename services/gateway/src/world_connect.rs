use std::{net::SocketAddr, any::Any};

use naia_serde::{BitReader, BitWriter};

use config::{
    WORLD_SERVER_RECV_ADDR,
    WORLD_SERVER_SIGNAL_PORT,
};
use http_client::ResponseError;
use http_server::{Method, Request, Response};
use logging::{info, warn};

use world_server_naia_proto::{
    messages::{FakeEntityConverter, Message, Auth as WorldAuth},
    protocol,
};

use crate::access_token_checker::middleware_impl;

pub(crate) async fn handler(
    _addr: SocketAddr,
    incoming_request: Request,
) -> Result<Response, ResponseError> {

    let host_name = "gateway";

    let world_server = "world_server";
    let remote_addr = WORLD_SERVER_RECV_ADDR;
    let remote_port = WORLD_SERVER_SIGNAL_PORT.to_string();
    let remote_method = Method::Post;
    let remote_path = "world_rtc";

    let logged_remote_url = format!(
        "{} host:{}/{}",
        remote_method.as_str(),
        remote_port,
        remote_path
    );

    http_server::http_log_util::send_req(host_name, world_server, &logged_remote_url);

    let world_auth = get_world_auth_from_header(&incoming_request).unwrap(); // if this fails, it means the auth middleware failed

    let world_auth_bytes = {
        let world_auth = WorldAuth::new(None, &world_auth.login_token);

        // TODO: this operation is VERY heavy! We should cache the result
        let message_kinds = protocol().into().message_kinds;

        let mut writer = BitWriter::new();
        world_auth.write(&message_kinds, &mut writer, &mut FakeEntityConverter);
        let bytes = writer.to_bytes();

        // base64 encode
        base64::encode(&bytes)
    };

    let mut world_rtc_request = incoming_request.clone();
    world_rtc_request.url = format!("http://{}:{}/{}", remote_addr, remote_port, remote_path);
    world_rtc_request.set_header("Authorization", &world_auth_bytes);
    match http_client::raw::fetch_async(world_rtc_request).await {
        Ok(world_rtc_response) => {
            http_server::http_log_util::recv_res(host_name, world_server, &logged_remote_url);
            return Ok(world_rtc_response);
        }
        Err(err) => {
            warn!(
                "Failed world_rtc request to world server: {}",
                err.to_string()
            );
            return Err(ResponseError::InternalServerError(
                "internal server error".to_string(),
            ));
        }
    }
}

pub(crate) async fn auth_middleware(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> Option<Result<Response, ResponseError>> {

    let access_token: Option<String> = get_world_auth_from_header(&incoming_request).map(|auth| auth.access_token).flatten();
    if access_token.is_some() {
        info!("found access_token in header: {}", access_token.as_ref().unwrap());
    } else {
        info!("no access_token found in header");
    }
    middleware_impl(incoming_addr, incoming_request, access_token).await
}

fn get_world_auth_from_header(incoming_request: &Request) -> Option<WorldAuth> {
    // info!("get_access_token_from_base64");
    let auth_header = incoming_request.get_header("authorization").map(|s| s.clone())?;
    // info!("auth_header: {}", auth_header);
    let auth_bytes = base64::decode(&auth_header).ok()?;
    // info!("auth_bytes: {:?}", auth_bytes);

    // TODO: this operation is VERY heavy! We should cache the result
    let message_kinds = protocol().into().message_kinds;
    let mut bit_reader = BitReader::new(&auth_bytes);
    let Ok(auth_message) = message_kinds.read(&mut bit_reader, &FakeEntityConverter) else {
        warn!("failed to read auth message from header");
        return None;
    };
    // info!("auth_message read from header");
    let auth_message_any = auth_message.clone().to_boxed_any();
    let auth_message: WorldAuth = Box::<dyn Any + 'static>::downcast::<WorldAuth>(auth_message_any)
        .ok()
        .map(|boxed_m| *boxed_m)
        .unwrap();
    Some(auth_message)
}