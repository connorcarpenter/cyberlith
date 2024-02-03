
use std::time::Instant;

use http_server::{async_dup::Arc, smol::lock::RwLock};

pub struct SessionInstance {
    http_addr: String,
    http_port: u16,
    signal_addr: String,
    signal_port: u16,
    last_heard: Arc<RwLock<Instant>>,
}

impl SessionInstance {
    pub fn new(http_addr: String, http_port: u16, signal_addr: String, signal_port: u16) -> Self {
        Self {
            http_addr,
            http_port,
            signal_addr,
            signal_port,
            last_heard: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub fn http_addr(&self) -> String {
        self.http_addr.clone()
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    pub fn signal_addr(&self) -> String {
        self.signal_addr.clone()
    }

    pub fn signal_port(&self) -> u16 {
        self.signal_port
    }

    pub fn last_heard(&self) -> Arc<RwLock<Instant>> {
        self.last_heard.clone()
    }
}

pub struct WorldInstance {
    http_addr: String,
    http_port: u16,
    signal_addr: String,
    signal_port: u16,
    last_heard: Arc<RwLock<Instant>>,
}

impl WorldInstance {
    pub fn new(http_addr: String, http_port: u16, signal_addr: String, signal_port: u16) -> Self {
        Self {
            http_addr,
            http_port,
            signal_addr,
            signal_port,
            last_heard: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub fn http_addr(&self) -> String {
        self.http_addr.clone()
    }

    pub fn http_port(&self) -> u16 {
        self.http_port
    }

    pub fn signal_addr(&self) -> String {
        self.signal_addr.clone()
    }

    pub fn signal_port(&self) -> u16 {
        self.signal_port
    }

    pub fn last_heard(&self) -> Arc<RwLock<Instant>> {
        self.last_heard.clone()
    }
}