use std::{collections::HashMap, time::Duration};

use http_client::HttpClient;
use http_server::{
    async_dup::Arc,
    executor::smol::{lock::RwLock, Timer},
    Server,
};
use logging::{info, warn};
use session_server_http_proto::{SocialLobbyPatch, SocialPatchGlobalChatMessagesRequest, SocialPatchMatchLobbiesRequest, SocialPatchUsersRequest, SocialUserPatch};
use config::SOCIAL_SERVER_GLOBAL_SECRET;

use crate::{
    match_lobbies::LobbyPatch, session_servers::SessionServerId, state::State, users::UserPatch,
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
            Timer::after(Duration::from_secs(1)).await;
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
