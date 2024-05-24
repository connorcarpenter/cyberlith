use std::collections::HashSet;
use std::time::Instant;

use http_server::{async_dup::Arc, executor::smol::lock::RwLock};

pub struct SocialInstance {
    http_addr: String,
    http_port: u16,
    last_heard: Arc<RwLock<Instant>>,
    // instance secret
    connected_session_servers: HashSet<String>,
}

impl SocialInstance {
    pub fn new(http_addr: &str, http_port: u16) -> Self {
        Self {
            http_addr: http_addr.to_string(),
            http_port,
            last_heard: Arc::new(RwLock::new(Instant::now())),
            connected_session_servers: HashSet::new(),
        }
    }

    pub fn http_addr(&self) -> &str {
        &self.http_addr
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    pub fn last_heard(&self) -> Arc<RwLock<Instant>> {
        self.last_heard.clone()
    }

    pub fn key(&self) -> (String, u16) {
        (self.http_addr.clone(), self.http_port)
    }

    pub fn insert_connected_session_server(&mut self, instance_secret: &str) {
        self.connected_session_servers.insert(instance_secret.to_string());
    }

    pub fn remove_connected_session_server(&mut self, instance_secret: &str) {
        self.connected_session_servers.remove(instance_secret);
    }

    pub fn has_connected_session_server(&self, instance_secret: &str) -> bool {
        self.connected_session_servers.contains(instance_secret)
    }

    pub fn clear_connected_session_servers(&mut self) -> Vec<String> {
        let mut connected_session_servers = Vec::new();
        for instance_secret in self.connected_session_servers.iter() {
            connected_session_servers.push(instance_secret.clone());
        }
        self.connected_session_servers.clear();
        connected_session_servers
    }
}
