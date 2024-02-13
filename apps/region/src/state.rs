use std::{collections::{HashMap, HashSet}, time::{Duration, Instant}};

use log::{info, warn};

use http_client::{HttpClient, RequestOptions};
use http_server::Server;

use session_server_http_proto::HeartbeatRequest as SessionHeartbeatRequest;
use world_server_http_proto::HeartbeatRequest as WorldHeartbeatRequest;
use asset_server_http_proto::HeartbeatRequest as AssetHeartbeatRequest;
use config::REGION_SERVER_SECRET;

use crate::instances::{AssetInstance, SessionInstance, WorldInstance};

pub struct State {
    timeout: Duration,
    session_instances: HashMap<(String, u16), SessionInstance>,
    world_instances: HashMap<(String, u16), WorldInstance>,
    asset_instance: Option<AssetInstance>,
    assetless_session_instances: HashSet<(String, u16)>,
}

impl State {
    pub fn new(timeout: Duration) -> Self {
        State {
            timeout,
            session_instances: HashMap::new(),
            world_instances: HashMap::new(),
            asset_instance: None,
            assetless_session_instances: HashSet::new(),
        }
    }

    pub fn register_session_instance(&mut self, instance: SessionInstance) {
        let key = (instance.http_addr(), instance.http_port());
        self.session_instances.insert(key.clone(), instance);
        self.assetless_session_instances.insert(key);
    }

    pub fn deregister_session_instance(&mut self, http_addr: &str, http_port: u16) {
        let key = (http_addr.to_string(), http_port);
        self.session_instances.remove(&key);
        self.assetless_session_instances.remove(&key);
    }

    pub fn deregister_asset_instance(&mut self) {
        self.asset_instance = None;

        for (key, port) in self.session_instances.iter() {
            if !self.assetless_session_instances.contains(key) {
                self.assetless_session_instances.insert(key.clone());

                // send disconnect asset server message to session instance
                todo!();
            }
        }
    }

    pub fn get_available_session_server(&self) -> Option<&SessionInstance> {
        self.session_instances.values().next()
    }

    pub fn register_world_instance(&mut self, instance: WorldInstance) {
        let key = (instance.http_addr(), instance.http_port());
        self.world_instances.insert(key, instance);
    }

    pub fn get_available_world_server(&self) -> Option<&WorldInstance> {
        self.world_instances.values().next()
    }

    pub fn register_asset_instance(&mut self, instance: AssetInstance) {
        self.asset_instance = Some(instance);
    }

    pub async fn send_heartbeats(&mut self) {

        let now = Instant::now();

        self.disconnect_unheard_instances(now).await;

        // send out heartbeats
        for instance in self.session_instances.values() {

            let instance_addr = instance.http_addr();
            let instance_port = instance.http_port();
            let last_heard = instance.last_heard();

            Server::spawn(async move {
                let request =  SessionHeartbeatRequest::new(REGION_SERVER_SECRET);
                let options = RequestOptions {
                    timeout_opt: Some(Duration::from_secs(1)),
                };
                let response = HttpClient::send_with_options(&instance_addr, instance_port, request, options).await;
                match response {
                    Ok(_) => {
                        info!("from {:?}:{} - session heartbeat success", instance_addr, instance_port);
                        let mut last_heard = last_heard.write().await;
                        *last_heard = Instant::now();
                    },
                    Err(err) => {
                        warn!("from {:?}:{} - session heartbeat failure: {}", instance_addr, instance_port, err.to_string());
                    }
                }
            });
        }

        for instance in self.world_instances.values() {

            let instance_addr = instance.http_addr();
            let instance_port = instance.http_port();
            
            let last_heard = instance.last_heard();

            Server::spawn(async move {
                let request =  WorldHeartbeatRequest::new(REGION_SERVER_SECRET);
                let options = RequestOptions {
                    timeout_opt: Some(Duration::from_secs(1)),
                };
                let response = HttpClient::send_with_options(&instance_addr, instance_port, request, options).await;
                match response {
                    Ok(_) => {
                        info!("from {:?}:{} - world heartbeat success", instance_addr, instance_port);
                        let mut last_heard = last_heard.write().await;
                        *last_heard = Instant::now();
                    },
                    Err(err) => {
                        warn!("from {:?}:{} - world heartbeat failure: {}", instance_addr, instance_port, err.to_string());
                    }
                }
            });
        }

        if let Some(instance) = self.asset_instance.as_ref() {

            let instance_addr = instance.http_addr();
            let instance_port = instance.http_port();
            let last_heard = instance.last_heard();

            Server::spawn(async move {
                let request =  AssetHeartbeatRequest::new(REGION_SERVER_SECRET);
                let options = RequestOptions {
                    timeout_opt: Some(Duration::from_secs(1)),
                };
                let response = HttpClient::send_with_options(&instance_addr, instance_port, request, options).await;
                match response {
                    Ok(_) => {
                        info!("from {:?}:{} - asset heartbeat success", instance_addr, instance_port);
                        let mut last_heard = last_heard.write().await;
                        *last_heard = Instant::now();
                    },
                    Err(err) => {
                        warn!("from {:?}:{} - asset heartbeat failure: {}", instance_addr, instance_port, err.to_string());
                    }
                }
            });
        }
    }

    pub async fn sync_asset_session_instances(&mut self) {
        if self.asset_instance.is_none() {
            return;
        }
        if self.assetless_session_instances.is_empty() {
            return;
        }

        for (session_ip, session_port) in self.assetless_session_instances.iter() {
            todo!();
        }

        self.assetless_session_instances.clear();
    }

    async fn disconnect_unheard_instances(&mut self, now: Instant) {
        // clean up instances that have disconnected

        let timeout = self.timeout;

        {
            let mut disconnected_instances = Vec::new();
            for (addr, instance) in self.session_instances.iter() {
                let last_heard = *instance.last_heard().read().await;
                let elapsed = now.duration_since(last_heard);
                if elapsed.as_secs() > timeout.as_secs() {
                    disconnected_instances.push(addr.clone());
                }
            }
            for (addr_ip, addr_port) in disconnected_instances {
                info!("session instance {:?}:{:?} disconnected", addr_ip, addr_port);
                self.deregister_session_instance(&addr_ip, addr_port);
            }
        }

        {
            let mut disconnected_instances = Vec::new();
            for (addr, instance) in self.world_instances.iter() {
                let last_heard = *instance.last_heard().read().await;
                let elapsed = now.duration_since(last_heard);
                if elapsed.as_secs() > timeout.as_secs() {
                    disconnected_instances.push(addr.clone());
                }
            }
            for addr in disconnected_instances {
                info!("world instance {:?} disconnected", addr);
                self.world_instances.remove(&addr);
            }
        }

        {
            if let Some(instance) = &self.asset_instance {
                let last_heard = *instance.last_heard().read().await;
                let elapsed = now.duration_since(last_heard);
                if elapsed.as_secs() > timeout.as_secs() {
                    info!("asset instance disconnected");
                    self.deregister_asset_instance();
                }
            }
        }
    }
}