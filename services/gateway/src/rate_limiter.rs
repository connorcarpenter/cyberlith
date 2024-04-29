
use std::{net::{IpAddr, SocketAddr}, time::Duration, collections::HashMap};

use http_server::{async_dup::Arc, smol::lock::RwLock, Request, Response, Server, RequestMiddlewareAction};
use instant::Instant;
use timequeue::TimeQueue;

pub(crate) struct GlobalRateLimiter {
    ip_map: HashMap<IpAddr, Arc<RwLock<PeerRateLimiter>>>,
    req_max: usize,
    window_duration: Duration,
}

impl GlobalRateLimiter {
    fn new(req_max: usize, window_duration: Duration) -> Self {
        Self {
            ip_map: HashMap::new(),
            req_max,
            window_duration,
        }
    }

    fn has_peer(&self, ip_addr: &IpAddr) -> bool {
        self.ip_map.contains_key(ip_addr)
    }

    fn get_peer(&self, ip_addr: &IpAddr) -> Arc<RwLock<PeerRateLimiter>> {
        self.ip_map.get(ip_addr).unwrap().clone()
    }

    fn create_peer(&mut self, ip_addr: &IpAddr) -> Arc<RwLock<PeerRateLimiter>> {
        let peer_rate_limiter = Arc::new(RwLock::new(PeerRateLimiter::new(self.req_max, self.window_duration)));
        self.ip_map.insert(*ip_addr, peer_rate_limiter.clone());
        peer_rate_limiter
    }

    pub(crate) async fn prune(&mut self) {
        let now = Instant::now();

        let mut ips_to_prune = Vec::new();
        for (ip, peer_rate_limiter) in self.ip_map.iter() {
            let mut peer_rate_limiter = peer_rate_limiter.write().await;
            if peer_rate_limiter.len(&now) == 0 {
                ips_to_prune.push(*ip);
            }
        }

        for ip_to_prune in ips_to_prune {
            self.ip_map.remove(&ip_to_prune);
        }
    }
}

struct PeerRateLimiter {
    requests: TimeQueue<()>,
    req_max: usize,
    window_duration: Duration,
}

impl PeerRateLimiter {
    fn new(req_max: usize, window_duration: Duration) -> Self {
        Self {
            requests: TimeQueue::new(),
            req_max,
            window_duration,
        }
    }

    fn handle_request(&mut self) -> Result<(), usize> {
        let now = Instant::now();

        self.clear_expired_requests(&now);

        if self.requests.len() > self.req_max {
            let next_request = self.requests.peek_entry().unwrap();

            let retry_after_secs = next_request.instant.until(&now).as_secs() as usize;

            return Err(retry_after_secs);
        } else {
            let mut expire_time = now;
            expire_time.add_millis(self.window_duration.as_millis() as u32);
            self.requests.add_item(expire_time, ());
            return Ok(());
        }
    }

    fn clear_expired_requests(&mut self, now: &Instant) {
        loop {
            if self.requests.pop_item(now).is_none() {
                break;
            }
        }
    }

    fn len(&mut self, now: &Instant) -> usize {
        self.clear_expired_requests(now);

        self.requests.len()
    }
}

pub fn add_middleware(server: &mut Server, req_max: usize, window_duration: Duration) -> Arc<RwLock<GlobalRateLimiter>> {

    let global_limiter = Arc::new(RwLock::new(GlobalRateLimiter::new(req_max, window_duration)));
    let output = global_limiter.clone();

    server.request_middleware(move |addr, req| {
        let global_limiter = global_limiter.clone();
        async move { handler(global_limiter, addr, req).await }
    });

    output
}

async fn handler(
    global_rate_limiter: Arc<RwLock<GlobalRateLimiter>>,
    incoming_addr: SocketAddr,
    incoming_request: Request,
) -> RequestMiddlewareAction {

    let incoming_ip = incoming_addr.ip();

    let mut peer_rate_limiter = None;

    {
        let global_rate_limiter = global_rate_limiter.read().await;
        if global_rate_limiter.has_peer(&incoming_ip) {
            peer_rate_limiter = Some(global_rate_limiter.get_peer(&incoming_ip));
        }
    }

    if peer_rate_limiter.is_none() {
        let mut global_rate_limiter = global_rate_limiter.write().await;
        peer_rate_limiter = Some(global_rate_limiter.create_peer(&incoming_ip));
    }

    let peer_rate_limiter = peer_rate_limiter.unwrap();
    let mut peer_rate_limiter = peer_rate_limiter.write().await;
    match peer_rate_limiter.handle_request() {
        Ok(()) => RequestMiddlewareAction::Continue(incoming_request),
        Err(retry_after_secs) => {
            RequestMiddlewareAction::Stop(Response::too_many_requests(&incoming_request.url, retry_after_secs))
        }
    }
}