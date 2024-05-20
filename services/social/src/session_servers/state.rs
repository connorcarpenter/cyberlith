use std::{collections::{HashMap}, net::SocketAddr};

use logging::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionServerId {
    val: u64,
}

impl SessionServerId {
    fn new(val: u64) -> Self {
        Self { val }
    }
}

pub struct SessionServersState {
    next_session_server_id: u64,
    addr_to_session_server_id: HashMap<(String, u16), SessionServerId>,
    instances: HashMap<SessionServerId, (String, u16)>,
}

impl SessionServersState {
    pub fn new() -> Self {
        Self {
            next_session_server_id: 0,
            addr_to_session_server_id: HashMap::new(),
            instances: HashMap::new(),
        }
    }

    pub fn next_session_id(&mut self) -> SessionServerId {
        let session_id = SessionServerId::new(self.next_session_server_id);
        self.next_session_server_id += 1;
        session_id
    }

    pub fn add_instance(&mut self, addr: &str, port: u16) {
        let key = (addr.to_string(), port);
        let id = self.next_session_id();
        self.instances.insert(id, key.clone());
        self.addr_to_session_server_id.insert(key, id);
    }

    pub fn remove_instance(&mut self, addr: &str, port: u16) {
        let key = (addr.to_string(), port);
        let id = self.addr_to_session_server_id.remove(&key).unwrap();
        self.instances.remove(&id);
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, SessionServerId, (String, u16)> {
        self.instances.iter()
    }

    pub fn get_session_server_id(&self, socket_addr: &SocketAddr) -> Option<SessionServerId> {

        info!("get_session_server_id: socket_addr: {:?}", socket_addr);
        info!("map: {:?}", self.addr_to_session_server_id);

        warn!("MUST IMPLEMENT! HOW DO WE CONVERT THIS??");

        return None;

        //self.addr_to_session_server_id.get(&(addr.to_string(), port)).copied()
    }
}
