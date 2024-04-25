use std::{net::SocketAddr};

use naia_serde::BitWriter;

use config::{SESSION_SERVER_RECV_ADDR, REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, SESSION_SERVER_SIGNAL_PORT};
use http_client::{HttpClient, ResponseError};
use http_server::{Method, Request, Response};
use logging::warn;

use region_server_http_proto::SessionConnectRequest;
use session_server_naia_proto::{messages::{FakeEntityConverter, Message}, protocol};

pub(crate) async fn session_rtc_endpoint_handler(
    args: (SocketAddr, Request),
) -> Result<Response, ResponseError> {
    let (_addr, incoming_request) = args;

    let host_name = "gateway";

    // call to region server with login request
    let connect_response = {
        let region_server = "region_server";
        let remote_addr = REGION_SERVER_RECV_ADDR;
        let remote_port = REGION_SERVER_PORT;
        let remote_method = Method::Post;
        let remote_path = "session/connect";

        let logged_remote_url = format!("{} host:{}/{}", remote_method.as_str(), remote_port, remote_path);
        http_server::http_log_util::send_req(
            host_name,
            region_server,
            &logged_remote_url
        );

        let Ok(connect_response) =
            HttpClient::send(&remote_addr, remote_port, SessionConnectRequest).await
            else {
                warn!("Failed session_connect request to region server");
                return Err(ResponseError::InternalServerError(
                    "failed session_connect to region server".to_string(),
                ));
            };

        http_server::http_log_util::recv_res(
            host_name,
            region_server,
            &logged_remote_url
        );

        connect_response
    };

    // forward original request to session server
    {
        let session_server = "session_server";
        let remote_addr = SESSION_SERVER_RECV_ADDR;
        let remote_port = SESSION_SERVER_SIGNAL_PORT.to_string();
        let remote_method = Method::Post;
        let remote_path = "session_rtc";

        let logged_remote_url = format!("{} host:{}/{}", remote_method.as_str(), remote_port, remote_path);
        http_server::http_log_util::send_req(
            host_name,
            session_server,
            &logged_remote_url
        );

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
        session_rtc_request.headers.insert("Authorization".to_string(), session_auth_bytes);
        match http_client::raw::fetch_async(session_rtc_request).await {
            Ok(session_rtc_response) => {
                http_server::http_log_util::recv_res(
                    host_name,
                    session_server,
                    &logged_remote_url
                );
                return Ok(session_rtc_response);
            }
            Err(err) => {
                warn!("Failed session_rtc request to session server: {}", err.to_string());
                return Err(ResponseError::InternalServerError("internal server error".to_string()));
            }
        }
    }
}
