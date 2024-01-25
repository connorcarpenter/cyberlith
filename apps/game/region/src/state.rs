use std::{collections::HashMap, net::SocketAddr};

use crate::instances::{SessionInstance, WorldInstance};

pub struct State {
    session_instances: HashMap<SocketAddr, SessionInstance>,
    world_instances: HashMap<SocketAddr, WorldInstance>,
}

impl Default for State {
    fn default() -> Self {
        State {
            session_instances: HashMap::new(),
            world_instances: HashMap::new(),
        }
    }
}

impl State {
    pub fn register_session_instance(&mut self, instance: SessionInstance) {
        self.session_instances.insert(instance.http_addr(), instance);
    }

    pub fn get_available_session_server(&self) -> &SessionInstance {
        self.session_instances.values().next().unwrap()
    }

    pub fn register_world_instance(&mut self, instance: WorldInstance) {
        self.world_instances.insert(instance.http_addr(), instance);
    }

    pub fn get_available_world_server(&self) -> &WorldInstance {
        self.world_instances.values().next().unwrap()
    }
}