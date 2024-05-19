use std::collections::{hash_set::Iter, HashSet};

pub struct SessionServersState {
    instances: HashSet<(String, u16)>,
}

impl SessionServersState {
    pub fn new() -> Self {
        Self {
            instances: HashSet::new(),
        }
    }

    pub fn add(&mut self, addr: &str, port: u16) {
        self.instances.insert((addr.to_string(), port));
    }

    pub fn remove(&mut self, addr: &str, port: u16) {
        self.instances.remove(&(addr.to_string(), port));
    }

    pub fn iter(&self) -> Iter<'_, (String, u16)> {
        self.instances.iter()
    }
}
