use std::collections::HashMap;

use auth_server_types::UserId;
use config::SOCIAL_SERVER_GLOBAL_SECRET;
use http_client::HttpClient;
use logging::{info, warn};
use session_server_http_proto::SocialPatchGlobalChatMessagesRequest;

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
        global_chat_full_log: Vec<(UserId, String)>,
    ) {
        let id = self.next_session_id();
        self.instances.insert(id, SessionInstance::new(recv_addr, recv_port));
        self.secret_to_session_server_id.insert(instance_secret.to_string(), id);

        // update with full state
        let request = SocialPatchGlobalChatMessagesRequest::new(SOCIAL_SERVER_GLOBAL_SECRET, global_chat_full_log);
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
                    recv_addr, recv_port, e.to_string()
                );
            }
        }
    }

    pub fn remove_instance(&mut self, session_secret: &str) {
        let id = self.secret_to_session_server_id.remove(session_secret).unwrap();
        self.instances.remove(&id);
    }

    pub fn get_session_server_id(&self, session_instance_secret: &str) -> Option<SessionServerId> {
        self.secret_to_session_server_id.get(session_instance_secret).copied()
    }

    pub fn all_session_ids(&self) -> Vec<SessionServerId> {
        self.instances.keys().copied().collect()
    }

    pub fn get_recv_addr(&self, session_server_id: SessionServerId) -> Option<(&str, u16)> {
        self.instances.get(&session_server_id).map(|instance| (instance.addr.as_str(), instance.port))
    }
}
