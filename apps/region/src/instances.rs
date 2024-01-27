
use std::{time::Instant, net::SocketAddr};

use http_server::{async_dup::Arc, smol::lock::RwLock};

pub struct SessionInstance {
    http_addr: SocketAddr,
    signal_addr: SocketAddr,
    last_heard: Arc<RwLock<Instant>>,
}

impl SessionInstance {
    pub fn new(http_addr: SocketAddr, signal_addr: SocketAddr) -> Self {
        Self {
            http_addr,
            signal_addr,
            last_heard: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub fn http_addr(&self) -> SocketAddr {
        self.http_addr
    }

    pub fn signal_addr(&self) -> SocketAddr {
        self.signal_addr
    }

    pub fn last_heard(&self) -> Arc<RwLock<Instant>> {
        self.last_heard.clone()
    }
}

pub struct WorldInstance {
    http_addr: SocketAddr,
    signal_addr: SocketAddr,
    last_heard: Arc<RwLock<Instant>>,
}

impl WorldInstance {
    pub fn new(http_addr: SocketAddr, signal_addr: SocketAddr) -> Self {
        Self {
            http_addr,
            signal_addr,
            last_heard: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub fn http_addr(&self) -> SocketAddr {
        self.http_addr
    }

    pub fn signal_addr(&self) -> SocketAddr {
        self.signal_addr
    }

    pub fn last_heard(&self) -> Arc<RwLock<Instant>> {
        self.last_heard.clone()
    }
}