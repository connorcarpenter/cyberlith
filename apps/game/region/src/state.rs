use std::{collections::HashMap, net::SocketAddr, time::{Duration, Instant}};

use log::{info, warn};

use http_client::HttpClient;
use http_server::Server;

use session_server_http_proto::HeartbeatRequest as SessionHeartbeatRequest;
use world_server_http_proto::HeartbeatRequest as WorldHeartbeatRequest;
use config::REGION_SERVER_SECRET;

use crate::instances::{SessionInstance, WorldInstance};

pub struct State {
    timeout: Duration,
    session_instances: HashMap<SocketAddr, SessionInstance>,
    world_instances: HashMap<SocketAddr, WorldInstance>,
}

impl State {
    pub fn new(timeout: Duration) -> Self {
        State {
            timeout,
            session_instances: HashMap::new(),
            world_instances: HashMap::new(),
        }
    }

    pub fn register_session_instance(&mut self, instance: SessionInstance) {
        self.session_instances.insert(instance.http_addr(), instance);
    }

    pub fn get_available_session_server(&self) -> Option<&SessionInstance> {
        self.session_instances.values().next()
    }

    pub fn register_world_instance(&mut self, instance: WorldInstance) {
        self.world_instances.insert(instance.http_addr(), instance);
    }

    pub fn get_available_world_server(&self) -> Option<&WorldInstance> {
        self.world_instances.values().next()
    }

    pub async fn send_heartbeats(&mut self) {

        let now = Instant::now();

        // clean up instances that have disconnected
        {

            let timeout = self.timeout;

            {
                let mut disconnected_instances = Vec::new();
                for (addr, instance) in self.session_instances.iter() {
                    let last_heard = *instance.last_heard().read().await;
                    let elapsed = now.duration_since(last_heard);
                    if elapsed.as_secs() > timeout.as_secs() {
                        disconnected_instances.push(*addr);
                    }
                }
                for addr in disconnected_instances {
                    info!("session instance {:?} disconnected", addr);
                    self.session_instances.remove(&addr);
                }
            }

            {
                let mut disconnected_instances = Vec::new();
                for (addr, instance) in self.world_instances.iter() {
                    let last_heard = *instance.last_heard().read().await;
                    let elapsed = now.duration_since(last_heard);
                    if elapsed.as_secs() > timeout.as_secs() {
                        disconnected_instances.push(*addr);
                    }
                }
                for addr in disconnected_instances {
                    info!("world instance {:?} disconnected", addr);
                    self.world_instances.remove(&addr);
                }
            }
        }

        // send out heartbeats
        for instance in self.session_instances.values() {

            let http_addr = instance.http_addr();
            let last_heard = instance.last_heard();

            Server::spawn(async move {
                let request =  SessionHeartbeatRequest::new(REGION_SERVER_SECRET);
                let response = HttpClient::send(&http_addr, request).await;
                match response {
                    Ok(_) => {
                        info!("from {:?} - session heartbeat success", http_addr);
                        let mut last_heard = last_heard.write().await;
                        *last_heard = Instant::now();
                    },
                    Err(err) => {
                        warn!("from {:?} - session heartbeat failure: {}", http_addr, err.to_string());
                    }
                }
            });
        }

        for instance in self.world_instances.values() {

            let http_addr = instance.http_addr();
            let last_heard = instance.last_heard();

            Server::spawn(async move {
                let request =  WorldHeartbeatRequest::new(REGION_SERVER_SECRET);
                let response = HttpClient::send(&http_addr, request).await;
                match response {
                    Ok(_) => {
                        info!("from {:?} - world heartbeat success", http_addr);
                        let mut last_heard = last_heard.write().await;
                        *last_heard = Instant::now();
                    },
                    Err(err) => {
                        warn!("from {:?} - world heartbeat failure: {}", http_addr, err.to_string());
                    }
                }
            });
        }
    }
}