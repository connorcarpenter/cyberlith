use std::time::{Duration, Instant};

use config::REGION_SERVER_SECRET;
use http_client::{HttpClient, RequestOptions};
use http_server::{ApiRequest, ApiResponse, Server};
use logging::{info, warn};

use asset_server_http_proto::{
    HeartbeatRequest as AssetHeartbeatRequest, HeartbeatResponse as AssetHeartbeatResponse,
};
use session_server_http_proto::{
    ConnectAssetServerRequest, ConnectAssetServerResponse, ConnectSocialServerRequest,
    ConnectSocialServerResponse, DisconnectAssetServerRequest, DisconnectAssetServerResponse,
    DisconnectSocialServerRequest, DisconnectSocialServerResponse,
    HeartbeatRequest as SessionHeartbeatRequest, HeartbeatResponse as SessionHeartbeatResponse,
};
use social_server_http_proto::{
    ConnectSessionServerRequest, ConnectSessionServerResponse, DisconnectSessionServerRequest,
    DisconnectSessionServerResponse, HeartbeatRequest as SocialHeartbeatRequest,
    HeartbeatResponse as SocialHeartbeatResponse,
};
use world_server_http_proto::{
    HeartbeatRequest as WorldHeartbeatRequest, HeartbeatResponse as WorldHeartbeatResponse,
};

use crate::{
    asset_instance::AssetInstance, session_instance::SessionInstance,
    social_instance::SocialInstance, world_instance::WorldInstance,
};

// Heartbeats
pub(crate) async fn send_session_heartbeat_request(session_instance: &SessionInstance) {
    let session_addr = session_instance.http_addr().to_string();
    let session_port = session_instance.http_port();
    let session_last_heard = session_instance.last_heard();

    Server::spawn(async move {
        let request = SessionHeartbeatRequest::new(REGION_SERVER_SECRET);
        let options = RequestOptions {
            timeout_opt: Some(Duration::from_secs(1)),
        };

        let host_name = "region";
        let remote_name = "session";
        http_server::log_util::send_req(host_name, remote_name, SessionHeartbeatRequest::name());
        let response =
            HttpClient::send_with_options(&session_addr, session_port, request, options).await;
        http_server::log_util::recv_res(host_name, remote_name, SessionHeartbeatResponse::name());

        match response {
            Ok(_) => {
                let mut last_heard = session_last_heard.write().await;
                *last_heard = Instant::now();
            }
            Err(err) => {
                warn!(
                    "from {:?}:{} - session heartbeat failure: {}",
                    session_addr,
                    session_port,
                    err.to_string()
                );
            }
        }
    });
}

pub(crate) async fn send_world_heartbeat_request(world_instance: &WorldInstance) {
    let world_addr = world_instance.http_addr().to_string();
    let world_port = world_instance.http_port();
    let world_last_heard = world_instance.last_heard().clone();

    Server::spawn(async move {
        let request = WorldHeartbeatRequest::new(REGION_SERVER_SECRET);
        let options = RequestOptions {
            timeout_opt: Some(Duration::from_secs(1)),
        };

        let host = "region";
        let remote = "world";
        http_server::log_util::send_req(host, remote, WorldHeartbeatRequest::name());
        let response =
            HttpClient::send_with_options(&world_addr, world_port, request, options).await;
        http_server::log_util::recv_res(host, remote, WorldHeartbeatResponse::name());

        match response {
            Ok(_) => {
                // info!(
                //     "from {:?}:{} - world heartbeat success",
                //     world_addr, world_port
                // );
                let mut last_heard = world_last_heard.write().await;
                *last_heard = Instant::now();
            }
            Err(err) => {
                warn!(
                    "from {:?}:{} - world heartbeat failure: {}",
                    world_addr,
                    world_port,
                    err.to_string()
                );
            }
        }
    });
}

pub(crate) async fn send_asset_heartbeat_request(asset_instance: &AssetInstance) {
    let asset_addr = asset_instance.http_addr().to_string();
    let asset_port = asset_instance.http_port();
    let asset_last_port = asset_instance.last_heard();

    Server::spawn(async move {
        let request = AssetHeartbeatRequest::new(REGION_SERVER_SECRET);
        let options = RequestOptions {
            timeout_opt: Some(Duration::from_secs(1)),
        };

        let host = "region";
        let remote = "asset";
        http_server::log_util::send_req(host, remote, AssetHeartbeatRequest::name());
        let response =
            HttpClient::send_with_options(&asset_addr, asset_port, request, options).await;
        http_server::log_util::recv_res(host, remote, AssetHeartbeatResponse::name());

        match response {
            Ok(_) => {
                // info!(
                //     "from {:?}:{} - asset heartbeat success",
                //     asset_addr, asset_port
                // );
                let mut last_heard = asset_last_port.write().await;
                *last_heard = Instant::now();
            }
            Err(err) => {
                warn!(
                    "from {:?}:{} - asset heartbeat failure: {}",
                    asset_addr,
                    asset_port,
                    err.to_string()
                );
            }
        }
    });
}

pub(crate) async fn send_social_heartbeat_request(social_instance: &SocialInstance) {
    let social_addr = social_instance.http_addr().to_string();
    let social_port = social_instance.http_port();
    let social_last_heard = social_instance.last_heard();

    Server::spawn(async move {
        let request = SocialHeartbeatRequest::new(REGION_SERVER_SECRET);
        let options = RequestOptions {
            timeout_opt: Some(Duration::from_secs(1)),
        };

        let host = "region";
        let remote = "social";
        http_server::log_util::send_req(host, remote, SocialHeartbeatRequest::name());
        let response =
            HttpClient::send_with_options(&social_addr, social_port, request, options).await;
        http_server::log_util::recv_res(host, remote, SocialHeartbeatResponse::name());

        match response {
            Ok(_) => {
                // info!(
                //     "from {:?}:{} - social heartbeat success",
                //     social_addr, social_port
                // );
                let mut last_heard = social_last_heard.write().await;
                *last_heard = Instant::now();
            }
            Err(err) => {
                warn!(
                    "from {:?}:{} - social heartbeat failure: {}",
                    social_addr,
                    social_port,
                    err.to_string()
                );
            }
        }
    });
}

// Others

pub(crate) async fn send_disconnect_session_server_message_to_social_instance(
    session_instance_secret: &str,
    social_instance: &SocialInstance,
) {
    // send disconnect session server message to social instance

    let session_instance_secret = session_instance_secret.to_string();

    let social_addr = social_instance.http_addr().to_string();
    let social_port = social_instance.http_port();
    let social_last_heard = social_instance.last_heard();

    Server::spawn(async move {
        let request =
            DisconnectSessionServerRequest::new(REGION_SERVER_SECRET, &session_instance_secret);

        let host = "region";
        let remote = "social";
        http_server::log_util::send_req(host, remote, DisconnectSessionServerRequest::name());
        let response = HttpClient::send(&social_addr, social_port, request).await;
        http_server::log_util::recv_res(host, remote, DisconnectSessionServerResponse::name());

        match response {
            Ok(_) => {
                // info!(
                //     "from {:?}:{} - social server disconnect session server success",
                //     social_instance_addr, social_instance_port
                // );
                let mut last_heard = social_last_heard.write().await;
                *last_heard = Instant::now();
            }
            Err(err) => {
                warn!(
                    "from {:?}:{} - social server disconnect session server failure: {}",
                    social_addr,
                    social_port,
                    err.to_string()
                );
            }
        }
    });
}

pub(crate) async fn send_disconnect_social_server_message_to_session_instance(
    session_instance: &SessionInstance,
) {
    // send disconnect social server message to session instance

    let session_addr = session_instance.http_addr().to_string();
    let session_port = session_instance.http_port();
    let last_heard = session_instance.last_heard();

    Server::spawn(async move {
        let request = DisconnectSocialServerRequest::new(REGION_SERVER_SECRET);

        let host_name = "region";
        let remote_name = "session";
        http_server::log_util::send_req(
            host_name,
            remote_name,
            DisconnectSocialServerRequest::name(),
        );
        let response = HttpClient::send(&session_addr, session_port, request).await;
        http_server::log_util::recv_res(
            host_name,
            remote_name,
            DisconnectSocialServerResponse::name(),
        );

        match response {
            Ok(_) => {
                // info!(
                //     "from {:?}:{} - session disconnect social server success",
                //     session_addr, session_port
                // );
                let mut last_heard = last_heard.write().await;
                *last_heard = Instant::now();
            }
            Err(err) => {
                warn!(
                    "from {:?}:{} - session disconnect social server failure: {}",
                    session_addr,
                    session_port,
                    err.to_string()
                );
            }
        }
    });
}

pub(crate) async fn send_connect_asset_server_req_to_session_server(
    asset_instance: &AssetInstance,
    session_instance: &SessionInstance,
) {
    let session_addr = session_instance.http_addr().to_string();
    let session_port = session_instance.http_port();
    let session_last_heard = session_instance.last_heard();

    let asset_addr = asset_instance.http_addr().to_string();
    let asset_port = asset_instance.http_port();

    Server::spawn(async move {
        let request = ConnectAssetServerRequest::new(REGION_SERVER_SECRET, &asset_addr, asset_port);

        let host = "region";
        let remote = "session";
        http_server::log_util::send_req(host, remote, ConnectAssetServerRequest::name());
        let response = HttpClient::send(&session_addr, session_port, request).await;
        http_server::log_util::recv_res(host, remote, ConnectAssetServerResponse::name());

        match response {
            Ok(_) => {
                // info!(
                //     "from {:?}:{} - session connect asset server success",
                //     session_addr, session_port
                // );
                let mut last_heard = session_last_heard.write().await;
                *last_heard = Instant::now();
            }
            Err(err) => {
                warn!(
                    "from {:?}:{} - session connect asset server failure: {}",
                    session_addr,
                    session_port,
                    err.to_string()
                );
            }
        }
    });
}

pub(crate) async fn send_connect_social_server_req_to_session_server(
    social_instance: &SocialInstance,
    session_instance: &SessionInstance,
) {
    let session_instance_addr_1 = session_instance.http_addr().to_string();

    let session_instance_port = session_instance.http_port();

    let session_last_heard = session_instance.last_heard();

    let social_server_addr_1 = social_instance.http_addr().to_string();
    let social_server_port = social_instance.http_port();

    // session server receives connection to social server
    Server::spawn(async move {
        let request = ConnectSocialServerRequest::new(
            REGION_SERVER_SECRET,
            &social_server_addr_1,
            social_server_port,
        );

        let host = "region";
        let remote = "session";
        http_server::log_util::send_req(host, remote, ConnectSocialServerRequest::name());
        let response =
            HttpClient::send(&session_instance_addr_1, session_instance_port, request).await;
        http_server::log_util::recv_res(host, remote, ConnectSocialServerResponse::name());

        match response {
            Ok(_) => {
                // info!(
                //     "from {:?}:{} - session server connect social server success",
                //     session_instance_addr_1, session_instance_port
                // );
                let mut last_heard = session_last_heard.write().await;
                *last_heard = Instant::now();
            }
            Err(err) => {
                warn!(
                    "from {:?}:{} - session server connect social server failure: {}",
                    session_instance_addr_1,
                    session_instance_port,
                    err.to_string()
                );
            }
        }
    });
}

pub(crate) async fn send_connect_session_server_req_to_social_server(
    session_instance: &SessionInstance,
    social_instance: &SocialInstance,
) {
    let session_addr = session_instance.http_addr().to_string();
    let session_port = session_instance.http_port();
    let session_secret = session_instance.instance_secret().to_string();

    let social_addr = social_instance.http_addr().to_string();
    let social_port = social_instance.http_port();
    let social_last_heard = social_instance.last_heard();

    // session server receives connection to social server
    Server::spawn(async move {
        let request = ConnectSessionServerRequest::new(
            REGION_SERVER_SECRET,
            &session_secret,
            &session_addr,
            session_port,
        );

        let host_name = "region";
        let remote_name = "social";
        http_server::log_util::send_req(
            host_name,
            remote_name,
            ConnectSessionServerRequest::name(),
        );
        let response = HttpClient::send(&social_addr, social_port, request).await;
        http_server::log_util::recv_res(
            host_name,
            remote_name,
            ConnectSessionServerResponse::name(),
        );

        match response {
            Ok(_) => {
                // info!(
                //     "from {:?}:{} - social server connect session server success",
                //     session_addr, session_port
                // );
                let mut last_heard = social_last_heard.write().await;
                *last_heard = Instant::now();
            }
            Err(err) => {
                warn!(
                    "from {:?}:{} - social server connect social session failure: {}",
                    session_addr,
                    session_port,
                    err.to_string()
                );
            }
        }
    });
}

pub(crate) async fn send_disconnect_asset_instance_to_session_instance(
    session_instance: &mut SessionInstance,
) {
    // send disconnect asset server message to session instance

    let session_addr = session_instance.http_addr().to_string();
    let session_port = session_instance.http_port();
    let last_heard = session_instance.last_heard();

    Server::spawn(async move {
        let request = DisconnectAssetServerRequest::new(REGION_SERVER_SECRET);

        let host = "region";
        let remote = "session";
        http_server::log_util::send_req(host, remote, DisconnectAssetServerRequest::name());
        let response = HttpClient::send(&session_addr, session_port, request).await;
        http_server::log_util::recv_res(host, remote, DisconnectAssetServerResponse::name());

        match response {
            Ok(_) => {
                // info!(
                //     "from {:?}:{} - session disconnect asset server success",
                //     session_addr, session_port
                // );
                let mut last_heard = last_heard.write().await;
                *last_heard = Instant::now();
            }
            Err(err) => {
                warn!(
                    "from {:?}:{} - session disconnect asset server failure: {}",
                    session_addr,
                    session_port,
                    err.to_string()
                );
            }
        }
    });
}
