
use std::time::Instant;

use http_server::{async_dup::Arc, smol::lock::RwLock};

pub struct SessionInstance {
    instance_secret: String,
    http_addr: String,
    http_port: u16,
    public_webrtc_url: String,
    last_heard: Arc<RwLock<Instant>>,
}

impl SessionInstance {
    pub fn new(instance_secret: &str, http_addr: &str, http_port: u16, public_webrtc_url: &str) -> Self {
        Self {
            instance_secret: instance_secret.to_string(),
            http_addr: http_addr.to_string(),
            http_port,
            public_webrtc_url: public_webrtc_url.to_string(),
            last_heard: Arc::new(RwLock::new(Instant::now())),
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

    pub fn public_webrtc_url(&self) -> &str {
        &self.public_webrtc_url
    }

    pub fn last_heard(&self) -> Arc<RwLock<Instant>> {
        self.last_heard.clone()
    }
}

pub struct WorldInstance {
    instance_secret: String,
    http_addr: String,
    http_port: u16,
    public_webrtc_url: String,
    last_heard: Arc<RwLock<Instant>>,
}

impl WorldInstance {
    pub fn new(instance_secret: &str, http_addr: &str, http_port: u16, public_webrtc_url: &str) -> Self {
        Self {
            instance_secret: instance_secret.to_string(),
            http_addr: http_addr.to_string(),
            http_port,
            public_webrtc_url: public_webrtc_url.to_string(),
            last_heard: Arc::new(RwLock::new(Instant::now())),
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

    pub fn public_webrtc_url(&self) -> &str {
        &self.public_webrtc_url
    }

    pub fn last_heard(&self) -> Arc<RwLock<Instant>> {
        self.last_heard.clone()
    }
}

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