use std::collections::{HashMap, HashSet};

use auth_server_types::UserId;
use config::SOCIAL_SERVER_GLOBAL_SECRET;
use http_client::HttpClient;
use logging::{info, warn};
use session_server_http_proto::{
    SocialLobbyPatch, SocialPatchGlobalChatMessagesRequest, SocialPatchMatchLobbiesRequest,
    SocialPatchUsersRequest, SocialUserPatch,
};
use social_server_types::{LobbyId, MessageId, Timestamp};

use crate::match_lobbies::LobbyState;

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
    instance_secret: String,
    connected_users: HashSet<UserId>,
}

impl SessionInstance {
    pub fn new(addr: &str, port: u16, instance_secret: &str) -> Self {
        Self {
            addr: addr.to_string(),
            port,
            instance_secret: instance_secret.to_string(),
            connected_users: HashSet::new(),
        }
    }

    pub fn has_user_connected(&self, user_id: &UserId) -> bool {
        self.connected_users.contains(user_id)
    }

    pub fn insert_connected_user(&mut self, user_id: UserId) {
        self.connected_users.insert(user_id);
    }

    pub fn remove_connected_user(&mut self, user_id: &UserId) {
        self.connected_users.remove(user_id);
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
        match_lobbies: Vec<(LobbyId, UserId, String, Vec<UserId>, LobbyState)>,
    ) {
        let id = self.next_session_id();
        self.instances
            .insert(id, SessionInstance::new(recv_addr, recv_port, instance_secret));
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
            let mut patches = Vec::new();

            for (lobby_id, owner_user_id, match_name, member_ids, state) in &match_lobbies {
                patches.push(SocialLobbyPatch::Create(*lobby_id, match_name.clone(), *owner_user_id));
                for member_id in member_ids {
                    patches.push(SocialLobbyPatch::Join(*lobby_id, *member_id));
                }
                if let LobbyState::InProgress = state {
                    patches.push(SocialLobbyPatch::Start(*lobby_id));
                }
            }

            let request =
                SocialPatchMatchLobbiesRequest::new(SOCIAL_SERVER_GLOBAL_SECRET, patches);
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

    pub fn get_session_instance_secret(&self, session_server_id: &SessionServerId) -> Option<&str> {
        self.instances
            .get(session_server_id)
            .map(|instance| instance.instance_secret.as_str())
    }

    pub fn all_session_ids(&self) -> Vec<SessionServerId> {
        self.instances.keys().copied().collect()
    }

    pub fn get_recv_addr(&self, session_server_id: SessionServerId) -> Option<(&str, u16)> {
        self.instances
            .get(&session_server_id)
            .map(|instance| (instance.addr.as_str(), instance.port))
    }

    pub(crate) fn session_server_user_connect(&mut self, session_server_id: &SessionServerId, user_id: &UserId) {
        self.instances
            .get_mut(session_server_id)
            .map(|instance| instance.insert_connected_user(*user_id));
    }

    pub(crate) fn session_server_user_disconnect(&mut self, session_server_id: &SessionServerId, user_id: &UserId) {
        self.instances
            .get_mut(session_server_id)
            .map(|instance| instance.remove_connected_user(user_id));
    }

    pub fn session_server_has_user_connected(&self, session_server_id: &SessionServerId, user_id: &UserId) -> bool {
        self.instances
            .get(session_server_id)
            .map(|instance| instance.has_user_connected(user_id))
            .unwrap_or(false)
    }
}
