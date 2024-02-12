use std::{collections::HashMap, time::{Duration, Instant}};

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
}

impl State {
    pub fn new(timeout: Duration) -> Self {
        State {
            timeout,
            session_instances: HashMap::new(),
            world_instances: HashMap::new(),
            asset_instance: None,
        }
    }

    pub fn register_session_instance(&mut self, instance: SessionInstance) {
        let key = (instance.http_addr(), instance.http_port());
        self.session_instances.insert(key, instance);
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

        // clean up instances that have disconnected
        {

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
                        self.asset_instance = None;
                    }
                }
            }
        }

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
}