use std::{collections::HashMap, net::SocketAddr};
use log::info;

use session_server_http_proto::HeartbeatRequest as SessionHeartbeatRequest;
use world_server_http_proto::HeartbeatRequest as WorldHeartbeatRequest;
use config::REGION_SERVER_SECRET;
use http_client::HttpClient;
use http_server::Server;

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

            let http_addr = instance.http_addr();

            Server::spawn(async move {
                let request =  SessionHeartbeatRequest::new(REGION_SERVER_SECRET);
                let response = HttpClient::send(&http_addr, request).await;
                match response {
                    Ok(_) => {
                        info!("session heartbeat success");
                    },
                    Err(_) => {
                        info!("session heartbeat failure");
                    }
                }
            });
        }

        for instance in self.world_instances.values() {

            let http_addr = instance.http_addr();

            Server::spawn(async move {
                let request =  WorldHeartbeatRequest::new(REGION_SERVER_SECRET);
                let response = HttpClient::send(&http_addr, request).await;
                match response {
                    Ok(_) => {
                        info!("world heartbeat success");
                    },
                    Err(_) => {
                        info!("world heartbeat failure");
                    }
                }
            });
        }
    }
}