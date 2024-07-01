use std::{thread, time::Duration};

use http_client::HttpClient;
use http_server::{async_dup::Arc, executor::smol::lock::RwLock, Server};
use logging::{info, warn};

use session_server_http_proto::SocialPatchGlobalChatMessagesRequest;

use config::SOCIAL_SERVER_GLOBAL_SECRET;

use crate::state::State;

pub fn start_processes(state: Arc<RwLock<State>>) {
    let state_clone = state.clone();
    Server::spawn(async move {
        loop {
            let state_clone_2 = state_clone.clone();
            send_patches(state_clone_2).await;
            thread::sleep(Duration::from_secs(5));
        }
    });
}

async fn send_patches(state: Arc<RwLock<State>>) {
    let state = &mut state.write().await;

    handle_global_chat_patches(state).await;
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
