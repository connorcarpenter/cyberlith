use std::net::SocketAddr;

use auth_server_types::UserId;
use config::{SOCIAL_SERVER_PORT, SOCIAL_SERVER_RECV_ADDR};
use http_client::HttpClient;
use http_server::{ApiRequest, ApiResponse, Request, RequestMiddlewareAction, Response};

use social_server_http_proto::{UserIsOnlineRequest, UserIsOnlineResponse};

pub(crate) async fn require_offline_or_redirect_home(
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> RequestMiddlewareAction {
    require_offline_or_redirect(incoming_addr, incoming_request, "/").await
}

async fn require_offline_or_redirect(
    incoming_addr: SocketAddr,
    incoming_request: Request,
    new_url: &str,
) -> RequestMiddlewareAction {
    let url = incoming_request.url.clone();
    match get_user_online_status(&incoming_addr, &incoming_request).await {
        UserPresenceResult::UserIsOffline => {
            // success
            RequestMiddlewareAction::Continue(incoming_request, None)
        }
        _ => {
            let response = Response::redirect(&url, new_url);
            RequestMiddlewareAction::Stop(response)
        }
    }
}

pub(crate) enum UserPresenceResult {
    UserIsOffline,
    UserIsOnline,
    ServerError,
    NoUserIdHeader,
    InvalidUserIdHeader,
}

async fn get_user_online_status(_incoming_addr: &SocketAddr, incoming_request: &Request) -> UserPresenceResult {

    let user_id = match incoming_request.get_header_first("user_id") {
        Some(user_id) => user_id,
        None => return UserPresenceResult::NoUserIdHeader,
    };
    let Ok(user_id) = user_id.parse::<u64>() else {
        return UserPresenceResult::InvalidUserIdHeader;
    };
    let user_id = UserId::new(user_id);

    return get_user_online_status_impl(user_id).await;
}

pub(crate) async fn get_user_online_status_impl(user_id: UserId) -> UserPresenceResult {
    let host_name = "gateway_demultiply";
    let social_server = "social_server";
    let social_addr = SOCIAL_SERVER_RECV_ADDR;
    let social_port = SOCIAL_SERVER_PORT;

    let request = UserIsOnlineRequest::new(user_id);

    http_server::log_util::send_req(
        host_name,
        social_server,
        UserIsOnlineRequest::name(),
    );
    let response_result =
        HttpClient::send(social_addr, social_port, request).await;
    http_server::log_util::recv_res(
        host_name,
        social_server,
        UserIsOnlineResponse::name(),
    );

    if let Ok(response) = response_result {
        if response.is_online() {
            return UserPresenceResult::UserIsOnline;
        } else {
            return UserPresenceResult::UserIsOffline;
        }
    } else {
        return UserPresenceResult::ServerError;
    }
}