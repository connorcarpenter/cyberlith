use std::{time::Duration, collections::HashMap};

use http_client::HttpClient;
use http_server::{
    async_dup::Arc,
    executor::smol::{lock::RwLock, Timer},
    ApiRequest, ApiResponse, Server,
};
use logging::{info, warn};
use config::{
    REGION_SERVER_PORT, REGION_SERVER_RECV_ADDR, SOCIAL_SERVER_GLOBAL_SECRET, SOCIAL_SERVER_PORT,
    SOCIAL_SERVER_RECV_ADDR,
};

use region_server_http_proto::{SocialRegisterInstanceRequest, SocialRegisterInstanceResponse};
use session_server_http_proto::SocialWorldConnectRequest;

use crate::{state::State, region::send_world_connect_request};

pub fn start_processes(state: Arc<RwLock<State>>) {
    let state_clone = state.clone();
    Server::spawn(async move {
        loop {
            send_register_instance_request(state_clone.clone()).await;
            process_region_server_disconnect(state_clone.clone()).await;
            handle_world_connect(state_clone.clone()).await;
            Timer::after(Duration::from_secs(1)).await;
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

async fn handle_world_connect(state: Arc<RwLock<State>>) {

    let state = &mut state.write().await;

    let starting_lobby_ids = state.match_lobbies.take_starting_lobbies();
    for starting_lobby_id in starting_lobby_ids {

        let lobby_user_ids = state.match_lobbies.get_lobby_users(&starting_lobby_id);

        // give notice to world server via region server, get tokens
        let mut req_data = HashMap::new();
        for user_id in lobby_user_ids {
            let session_server_id = state
                .users
                .get_user_session_server_id(&user_id);
            if !req_data.contains_key(&session_server_id) {
                req_data.insert(session_server_id, Vec::new());
            }
            let session_server = req_data.get_mut(&session_server_id).unwrap();
            session_server.push(user_id);
        }
        let mut req_data_2 = Vec::new();
        for (session_server_id, user_id) in req_data {
            let instance_secret = state
                .session_servers
                .get_session_instance_secret(&session_server_id)
                .unwrap()
                .to_string();
            req_data_2.push((instance_secret, user_id));
        }

        let (world_server_instance_secret, login_tokens) = match send_world_connect_request(
            &mut state.region_server,
            starting_lobby_id,
            req_data_2,
        ).await {
            Ok((world_server_instance_secret, login_tokens)) => {
                (world_server_instance_secret, login_tokens)
            }
            Err(err) => {
                warn!("failed to send world connect request: {:?}", err.to_string());
                return;
            }
        };

        let mut session_servers = HashMap::new();
        for (user_id, login_token) in login_tokens {
            let session_server_id = state
                .users
                .get_user_session_server_id(&user_id);
            if !state.session_servers.session_server_has_user_connected(&session_server_id, &user_id) {
                warn!("session server does not have user! should not be possible.");
                continue;
            }
            if !session_servers.contains_key(&session_server_id) {
                session_servers.insert(session_server_id, Vec::new());
            }
            let session_server = session_servers.get_mut(&session_server_id).unwrap();
            session_server.push((user_id, login_token));
        }

        // send world connect deets to all session servers
        for (session_server_id, outgoing_message) in session_servers {

            let (recv_addr, recv_port) = state
                .session_servers
                .get_recv_addr(session_server_id)
                .unwrap();

            let request = SocialWorldConnectRequest::new(
                SOCIAL_SERVER_GLOBAL_SECRET,
                &world_server_instance_secret,
                starting_lobby_id,
                outgoing_message
            );
            let response = HttpClient::send(recv_addr, recv_port, request).await;
            match response {
                Ok(_) => {
                    info!(
                        "from {:?}:{} - world connect sent",
                        recv_addr, recv_port
                    );
                }
                Err(e) => {
                    warn!(
                        "from {:?}:{} - world connect send failed: {:?}",
                        recv_addr,
                        recv_port,
                        e.to_string()
                    );
                }
            }
        }
    }
}