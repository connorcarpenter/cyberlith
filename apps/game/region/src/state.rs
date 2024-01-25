use std::{collections::HashMap, net::SocketAddr};
use log::info;

use session_server_http_proto::HeartbeatRequest as SessionHeartbeatRequest;
use world_server_http_proto::HeartbeatRequest as WorldHeartbeatRequest;
use config::REGION_SERVER_SECRET;
use http_client::HttpClient;

use crate::instances::{SessionInstance, WorldInstance};

pub struct State {
    session_instances: HashMap<SocketAddr, SessionInstance>,
    world_instances: HashMap<SocketAddr, WorldInstance>,
}

impl Default for State {
    fn default() -> Self {
        State {
            session_instances: HashMap::new(),
            world_instances: HashMap::new(),
        }
    }
}

impl State {
    pub fn register_session_instance(&mut self, instance: SessionInstance) {
        self.session_instances.insert(instance.http_addr(), instance);
    }

    pub fn get_available_session_server(&self) -> &SessionInstance {
        self.session_instances.values().next().unwrap()
    }

    pub fn register_world_instance(&mut self, instance: WorldInstance) {
        self.world_instances.insert(instance.http_addr(), instance);
    }

    pub fn get_available_world_server(&self) -> &WorldInstance {
        self.world_instances.values().next().unwrap()
    }

    pub async fn send_heartbeats(&self) {
        for instance in self.session_instances.values() {
            let request =  SessionHeartbeatRequest::new(REGION_SERVER_SECRET);
            let response = HttpClient::send(&instance.http_addr(), request).await;
            match response {
                Ok(_) => {
                    info!("heartbeat success");
                },
                Err(_) => {
                    info!("heartbeat failure");
                }
            }
        }

        for instance in self.world_instances.values() {
            let request =  WorldHeartbeatRequest::new(REGION_SERVER_SECRET);
            let response = HttpClient::send(&instance.http_addr(), request).await;
            match response {
                Ok(_) => {
                    info!("heartbeat success");
                },
                Err(_) => {
                    info!("heartbeat failure");
                }
            }
        }
    }
}