use std::collections::HashMap;

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

    pub fn next_session_id(&mut self) -> SessionServerId {
        let session_id = SessionServerId::new(self.next_session_server_id);
        self.next_session_server_id += 1;
        session_id
    }

    pub fn add_instance(&mut self, instance_secret: &str, addr: &str, port: u16) {
        let id = self.next_session_id();
        self.instances.insert(id, SessionInstance::new(addr, port));
        self.secret_to_session_server_id.insert(instance_secret.to_string(), id);
    }

    pub fn remove_instance(&mut self, session_secret: &str) {
        let id = self.secret_to_session_server_id.remove(session_secret).unwrap();
        self.instances.remove(&id);
    }

    pub fn get_session_server_id(&self, session_instance_secret: &str) -> Option<SessionServerId> {
        self.secret_to_session_server_id.get(session_instance_secret).copied()
    }
}
