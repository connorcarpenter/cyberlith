use std::{collections::HashMap, time::Duration};

use http_client::HttpClient;
use http_server::{
    async_dup::Arc,
    executor::smol::{lock::RwLock, Timer},
    Server,
};
use logging::{info, warn};
use session_server_http_proto::{SocialLobbyPatch, SocialPatchGlobalChatMessagesRequest, SocialPatchMatchLobbiesRequest, SocialPatchUsersRequest, SocialUserPatch, SocialWorldConnectRequest};
use config::SOCIAL_SERVER_GLOBAL_SECRET;

use crate::{
    match_lobbies::LobbyPatch, session_servers::SessionServerId, state::State, users::UserPatch, region::send_world_connect_request,
};

pub fn start_processes(state: Arc<RwLock<State>>) {
    let state_clone_1 = state.clone();

    // patches
    Server::spawn(async move {
        loop {
            let state = &mut state_clone_1.write().await;
            handle_user_patches(state).await;
            handle_global_chat_patches(state).await;
            handle_match_lobby_patches(state).await;
            handle_world_connect(state).await;
            Timer::after(Duration::from_secs(5)).await;
        }
    });
}

async fn handle_user_patches(state: &mut State) {
    let user_patches = state.users.take_patches();
    let mut queued_social_user_patches: HashMap<SessionServerId, Vec<SocialUserPatch>> =
        HashMap::new();
    let session_server_ids = state.session_servers.all_session_ids();

    for user_patch in user_patches {
        match user_patch {
            UserPatch::Add(user_id) => {
                for receiving_session_server_id in &session_server_ids {
                    let social_user_patch = SocialUserPatch::Add(user_id);
                    queued_social_user_patches
                        .entry(*receiving_session_server_id)
                        .or_insert(Vec::new())
                        .push(social_user_patch);
                }
            }
            UserPatch::Remove(sending_session_server_id, user_id) => {
                for receiving_session_server_id in &session_server_ids {
                    if sending_session_server_id == *receiving_session_server_id {
                        continue;
                    }

                    let social_user_patch = SocialUserPatch::Remove(user_id);
                    queued_social_user_patches
                        .entry(*receiving_session_server_id)
                        .or_insert(Vec::new())
                        .push(social_user_patch);
                }
            }
        }
    }

    for (receiving_session_server_id, user_patches) in queued_social_user_patches {
        let (recv_addr, recv_port) = state
            .session_servers
            .get_recv_addr(receiving_session_server_id)
            .unwrap();

        let request = SocialPatchUsersRequest::new(SOCIAL_SERVER_GLOBAL_SECRET, user_patches);
        let response = HttpClient::send(recv_addr, recv_port, request).await;
        match response {
            Ok(_) => {
                info!("from {:?}:{} - user patches sent", recv_addr, recv_port);
            }
            Err(e) => {
                warn!(
                    "from {:?}:{} - user patches send failed: {:?}",
                    recv_addr,
                    recv_port,
                    e.to_string()
                );
            }
        }
    }
}

async fn handle_global_chat_patches(state: &mut State) {
    let global_chat_patches = state.global_chat.take_patches();
    for (sending_session_server_id, messages) in global_chat_patches {
        for receiving_session_server_id in state.session_servers.all_session_ids() {
            if sending_session_server_id == receiving_session_server_id {
                continue;
            }

            let (recv_addr, recv_port) = state
                .session_servers
                .get_recv_addr(receiving_session_server_id)
                .unwrap();

            let request = SocialPatchGlobalChatMessagesRequest::new(
                SOCIAL_SERVER_GLOBAL_SECRET,
                messages.clone(),
            );
            let response = HttpClient::send(recv_addr, recv_port, request).await;
            match response {
                Ok(_) => {
                    info!(
                        "from {:?}:{} - global chat patch messages sent",
                        recv_addr, recv_port
                    );
                }
                Err(e) => {
                    warn!(
                        "from {:?}:{} - global chat patch messages send failed: {:?}",
                        recv_addr,
                        recv_port,
                        e.to_string()
                    );
                }
            }
        }
    }
}

async fn handle_match_lobby_patches(state: &mut State) {
    let match_lobby_patches = state.match_lobbies.take_patches();
    for (sending_session_server_id, patches) in match_lobby_patches {
        for receiving_session_server_id in state.session_servers.all_session_ids() {
            if sending_session_server_id == receiving_session_server_id {
                continue;
            }

            let (recv_addr, recv_port) = state
                .session_servers
                .get_recv_addr(receiving_session_server_id)
                .unwrap();

            let patches = patches
                .iter()
                .map(|patch| match patch {
                    LobbyPatch::Create(lobby_id, creator_user_id, match_name) => {
                        SocialLobbyPatch::Create(
                            lobby_id.clone(),
                            match_name.clone(),
                            creator_user_id.clone(),
                        )
                    }
                    LobbyPatch::Join(lobby_id, user_id) => SocialLobbyPatch::Join(
                        lobby_id.clone(),
                        user_id.clone(),
                    ),
                    LobbyPatch::Leave(user_id) => SocialLobbyPatch::Leave(user_id.clone()),
                    LobbyPatch::Message(message_id, timestamp, user_id, message) => SocialLobbyPatch::Message(
                        message_id.clone(),
                        timestamp.clone(),
                        user_id.clone(),
                        message.clone(),
                    ),
                    LobbyPatch::Start(lobby_id) => SocialLobbyPatch::Start(lobby_id.clone()),
                })
                .collect();

            let request = SocialPatchMatchLobbiesRequest::new(SOCIAL_SERVER_GLOBAL_SECRET, patches);
            let response = HttpClient::send(recv_addr, recv_port, request).await;
            match response {
                Ok(_) => {
                    info!(
                        "from {:?}:{} - match lobbies patches sent",
                        recv_addr, recv_port
                    );
                }
                Err(e) => {
                    warn!(
                        "from {:?}:{} - match lobbies patches send failed: {:?}",
                        recv_addr,
                        recv_port,
                        e.to_string()
                    );
                }
            }
        }
    }
}

async fn handle_world_connect(state: &mut State) {
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

            let request = SocialWorldConnectRequest::new(SOCIAL_SERVER_GLOBAL_SECRET, &world_server_instance_secret, outgoing_message);
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
