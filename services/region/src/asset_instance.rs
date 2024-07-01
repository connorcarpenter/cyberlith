use std::time::Instant;

use http_server::{async_dup::Arc, executor::smol::lock::RwLock};

pub struct AssetInstance {
    http_addr: String,
    http_port: u16,
    last_heard: Arc<RwLock<Instant>>,
}

impl AssetInstance {
    pub fn new(http_addr: &str, http_port: u16) -> Self {
        Self {
            http_addr: http_addr.to_string(),
            http_port,
            last_heard: Arc::new(RwLock::new(Instant::now())),
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
}
