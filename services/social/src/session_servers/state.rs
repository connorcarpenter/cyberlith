use std::collections::HashMap;

use auth_server_types::UserId;
use config::SOCIAL_SERVER_GLOBAL_SECRET;
use http_client::HttpClient;
use logging::{info, warn};
use session_server_http_proto::{
    SocialLobbyPatch, SocialPatchGlobalChatMessagesRequest, SocialPatchMatchLobbiesRequest,
    SocialPatchUsersRequest, SocialUserPatch,
};
use social_server_types::{LobbyId, MessageId, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionServerId {
    val: u64,
}

impl SessionServerId {
    fn new(val: u64) -> Self {
        Self { val }
    }
}

struct SessionInstance {
    addr: String,
    port: u16,
}

impl SessionInstance {
    pub fn new(addr: &str, port: u16) -> Self {
        Self {
            addr: addr.to_string(),
            port,
        }
    }
}

pub struct SessionServersState {
    next_session_server_id: u64,
    secret_to_session_server_id: HashMap<String, SessionServerId>,
    instances: HashMap<SessionServerId, SessionInstance>,
}

impl SessionServersState {
    pub fn new() -> Self {
        Self {
            next_session_server_id: 0,
            secret_to_session_server_id: HashMap::new(),
            instances: HashMap::new(),
        }
    }

    fn next_session_id(&mut self) -> SessionServerId {
        let session_id = SessionServerId::new(self.next_session_server_id);
        self.next_session_server_id += 1;
        session_id
    }

    pub async fn init_instance(
        &mut self,
        instance_secret: &str,
        recv_addr: &str,
        recv_port: u16,
        present_users: Vec<UserId>,
        global_chat_full_log: Vec<(MessageId, Timestamp, UserId, String)>,
        match_lobbies: Vec<(LobbyId, UserId, String)>,
    ) {
        let id = self.next_session_id();
        self.instances
            .insert(id, SessionInstance::new(recv_addr, recv_port));
        self.secret_to_session_server_id
            .insert(instance_secret.to_string(), id);

        // update with full state

        // sync users
        {
            let user_patches = present_users
                .iter()
                .map(|user_id| SocialUserPatch::Add(*user_id))
                .collect();
            let request = SocialPatchUsersRequest::new(SOCIAL_SERVER_GLOBAL_SECRET, user_patches);
            let response = HttpClient::send(recv_addr, recv_port, request).await;
            match response {
                Ok(_) => {
                    info!(
                        "from {:?}:{} - present users init req sent",
                        recv_addr, recv_port
                    );
                }
                Err(e) => {
                    warn!(
                        "from {:?}:{} - present users init req send failed: {:?}",
                        recv_addr,
                        recv_port,
                        e.to_string()
                    );
                }
            }
        }

        // sync global chat
        {
            let request = SocialPatchGlobalChatMessagesRequest::new(
                SOCIAL_SERVER_GLOBAL_SECRET,
                global_chat_full_log,
            );
            let response = HttpClient::send(recv_addr, recv_port, request).await;
            match response {
                Ok(_) => {
                    info!(
                        "from {:?}:{} - global chat init messages sent",
                        recv_addr, recv_port
                    );
                }
                Err(e) => {
                    warn!(
                        "from {:?}:{} - global chat init messages send failed: {:?}",
                        recv_addr,
                        recv_port,
                        e.to_string()
                    );
                }
            }
        }

        // sync match lobbies
        {
            let match_lobbies = match_lobbies
                .iter()
                .map(|(lobby_id, user_id, match_name)| {
                    SocialLobbyPatch::Create(*lobby_id, match_name.clone(), *user_id)
                })
                .collect();
            let request =
                SocialPatchMatchLobbiesRequest::new(SOCIAL_SERVER_GLOBAL_SECRET, match_lobbies);
            let response = HttpClient::send(recv_addr, recv_port, request).await;
            match response {
                Ok(_) => {
                    info!(
                        "from {:?}:{} - match lobbies init req sent",
                        recv_addr, recv_port
                    );
                }
                Err(e) => {
                    warn!(
                        "from {:?}:{} - match lobbies init req send failed: {:?}",
                        recv_addr,
                        recv_port,
                        e.to_string()
                    );
                }
            }
        }
    }

    pub fn remove_instance(&mut self, session_secret: &str) {
        let id = self
            .secret_to_session_server_id
            .remove(session_secret)
            .unwrap();
        self.instances.remove(&id);
    }

    pub fn clear(&mut self) {
        self.secret_to_session_server_id.clear();
        self.instances.clear();
    }

    pub fn get_session_server_id(&self, session_instance_secret: &str) -> Option<SessionServerId> {
        self.secret_to_session_server_id
            .get(session_instance_secret)
            .copied()
    }

    pub fn all_session_ids(&self) -> Vec<SessionServerId> {
        self.instances.keys().copied().collect()
    }

    pub fn get_recv_addr(&self, session_server_id: SessionServerId) -> Option<(&str, u16)> {
        self.instances
            .get(&session_server_id)
            .map(|instance| (instance.addr.as_str(), instance.port))
    }
}
