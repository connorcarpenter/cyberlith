use std::{thread, time::Duration};

use http_client::HttpClient;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, ApiRequest, ApiResponse, Server};
use logging::{info, warn};

use region_server_http_proto::{SocialRegisterInstanceRequest, SocialRegisterInstanceResponse};

use config::{
    REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR, SOCIAL_SERVER_GLOBAL_SECRET, SOCIAL_SERVER_PORT,
    SOCIAL_SERVER_RECV_ADDR,
};

use crate::state::State;

pub fn start_processes(state: Arc<RwLock<State>>) {
    // send registration
    let state_clone = state.clone();
    Server::spawn(async move {
        loop {
            let state_clone_2 = state_clone.clone();
            send_register_instance_request(state_clone_2).await;
            thread::sleep(Duration::from_secs(5));
        }
    });

    // handle disconnection
    let state_clone = state.clone();
    Server::spawn(async move {
        loop {
            let state_clone_2 = state_clone.clone();
            process_region_server_disconnect(state_clone_2).await;
            thread::sleep(Duration::from_secs(5));
        }
    });
}

async fn send_register_instance_request(state: Arc<RwLock<State>>) {
    let state = &mut state.write().await.region_server;

    if state.connected() {
        return;
    }
    if !state.time_to_resend_registration() {
        return;
    }

    let request = SocialRegisterInstanceRequest::new(
        SOCIAL_SERVER_GLOBAL_SECRET,
        SOCIAL_SERVER_RECV_ADDR,
        SOCIAL_SERVER_PORT,
    );

    let host = "social";
    let remote = "region";
    http_server::log_util::send_req(host, remote, SocialRegisterInstanceRequest::name());
    let response = HttpClient::send(REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT, request).await;
    http_server::log_util::recv_res(host, remote, SocialRegisterInstanceResponse::name());

    match response {
        Ok(_) => {
            // info!(
            //     "from {:?}:{} - social server registration success",
            //     REGION_SERVER_RECV_ADDR, REGION_SERVER_PORT
            // );
            state.set_connected();
        }
        Err(err) => {
            warn!(
                "from {:?}:{} - social server registration failure: {}",
                REGION_SERVER_RECV_ADDR,
                REGION_SERVER_PORT,
                err.to_string()
            );
        }
    }

    state.sent_to_region_server();
}

async fn process_region_server_disconnect(state: Arc<RwLock<State>>) {
    let state = &mut state.write().await;

    if state.region_server.connected() {
        if state.region_server.time_to_disconnect() {
            info!("disconnecting from region server");
            state.region_server.set_disconnected();

            // disconnect from session servers
            state.session_servers.clear();
        }
    }
}
