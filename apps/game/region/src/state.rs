use std::{collections::HashMap, net::SocketAddr};

use crate::instances::SessionInstance;

pub struct State {
    session_instances: HashMap<SocketAddr, SessionInstance>,
}

impl Default for State {
    fn default() -> Self {
        State {
            session_instances: HashMap::new()
        }
    }
}

impl State {
    pub fn register_session_instance(&mut self, incoming_addr: SocketAddr, instance: SessionInstance) {
        self.session_instances.insert(incoming_addr, instance);
    }

    pub fn get_available_session_server(&self) -> &SessionInstance {
        self.session_instances.values().next().unwrap()
    }
}