
use std::time::Instant;

use http_server::{async_dup::Arc, smol::lock::RwLock};

pub struct SessionInstance {
    http_addr: String,
    http_port: u16,
    public_url: String,
    last_heard: Arc<RwLock<Instant>>,
}

impl SessionInstance {
    pub fn new(http_addr: String, http_port: u16, public_url: String) -> Self {
        Self {
            http_addr,
            http_port,
            public_url,
            last_heard: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub fn http_addr(&self) -> String {
        self.http_addr.clone()
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    pub fn public_url(&self) -> String {
        self.public_url.clone()
    }

    pub fn last_heard(&self) -> Arc<RwLock<Instant>> {
        self.last_heard.clone()
    }
}

pub struct WorldInstance {
    http_addr: String,
    http_port: u16,
    public_url: String,
    last_heard: Arc<RwLock<Instant>>,
}

impl WorldInstance {
    pub fn new(http_addr: String, http_port: u16, public_url: String) -> Self {
        Self {
            http_addr,
            http_port,
            public_url,
            last_heard: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub fn http_addr(&self) -> String {
        self.http_addr.clone()
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    pub fn public_url(&self) -> String {
        self.public_url.clone()
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
    pub fn new(http_addr: String, http_port: u16) -> Self {
        Self {
            http_addr,
            http_port,
            last_heard: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub fn http_addr(&self) -> String {
        self.http_addr.clone()
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    pub fn last_heard(&self) -> Arc<RwLock<Instant>> {
        self.last_heard.clone()
    }
}