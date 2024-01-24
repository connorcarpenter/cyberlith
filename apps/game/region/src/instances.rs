use std::net::SocketAddr;

pub struct SessionInstance {
    http_addr: SocketAddr,
    signal_addr: SocketAddr,
}

impl SessionInstance {
    pub fn new(http_addr: SocketAddr, signal_addr: SocketAddr) -> Self {
        Self {
            http_addr,
            signal_addr,
        }
    }

    pub fn http_addr(&self) -> SocketAddr {
        self.http_addr
    }

    pub fn signal_addr(&self) -> SocketAddr {
        self.signal_addr
    }
}

pub struct WorldInstance {
    http_addr: SocketAddr,
    signal_addr: SocketAddr,
}

impl WorldInstance {
    pub fn new(http_addr: SocketAddr, signal_addr: SocketAddr) -> Self {
        Self {
            http_addr,
            signal_addr,
        }
    }

    pub fn http_addr(&self) -> SocketAddr {
        self.http_addr
    }

    pub fn signal_addr(&self) -> SocketAddr {
        self.signal_addr
    }
}