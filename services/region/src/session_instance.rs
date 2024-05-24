use std::time::Instant;

use http_server::{async_dup::Arc, executor::smol::lock::RwLock};

pub struct SessionInstance {
    instance_secret: String,
    http_addr: String,
    http_port: u16,
    last_heard: Arc<RwLock<Instant>>,
    has_asset_server: bool,
    has_social_server: bool,
}

impl SessionInstance {
    pub fn new(instance_secret: &str, http_addr: &str, http_port: u16) -> Self {
        Self {
            instance_secret: instance_secret.to_string(),
            http_addr: http_addr.to_string(),
            http_port,
            last_heard: Arc::new(RwLock::new(Instant::now())),
            has_asset_server: false,
            has_social_server: false,
        }
    }

    pub fn instance_secret(&self) -> &str {
        &self.instance_secret
    }

    pub fn http_addr(&self) -> &str {
        &self.http_addr
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    pub fn key(&self) -> (String, u16) {
        (self.http_addr.clone(), self.http_port)
    }

    pub fn last_heard(&self) -> Arc<RwLock<Instant>> {
        self.last_heard.clone()
    }

    pub fn has_asset_server(&self) -> bool {
        self.has_asset_server
    }

    pub fn has_social_server(&self) -> bool {
        self.has_social_server
    }

    pub(crate) fn set_has_asset_server(&mut self) {
        self.has_asset_server = true;
    }

    pub(crate) fn set_has_social_server(&mut self) {
        self.has_social_server = true;
    }

    pub fn clear_asset_server(&mut self) {
        self.has_asset_server = false;
    }

    pub fn clear_social_server(&mut self) {
        self.has_social_server = false;
    }
}