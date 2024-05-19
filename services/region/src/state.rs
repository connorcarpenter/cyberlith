use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use logging::{info, warn};
use http_client::{HttpClient, RequestOptions};
use http_server::Server;
use config::REGION_SERVER_SECRET;

use social_server_http_proto::{ConnectSessionServerRequest, DisconnectSessionServerRequest, HeartbeatRequest as SocialHeartbeatRequest};
use asset_server_http_proto::HeartbeatRequest as AssetHeartbeatRequest;
use session_server_http_proto::{ConnectAssetServerRequest, ConnectSocialServerRequest, DisconnectAssetServerRequest, DisconnectSocialServerRequest, HeartbeatRequest as SessionHeartbeatRequest};
use world_server_http_proto::HeartbeatRequest as WorldHeartbeatRequest;

use crate::instances::{AssetInstance, SessionInstance, SocialInstance, WorldInstance};

pub struct State {
    disconnect_timeout: Duration,

    session_instances: HashMap<(String, u16), SessionInstance>,
    world_instances: HashMap<(String, u16), WorldInstance>,
    asset_instance: Option<AssetInstance>,
    social_instance: Option<SocialInstance>,

    assetless_session_instances: HashSet<(String, u16)>,
    socialless_session_instances: HashSet<(String, u16)>,
}

impl State {
    pub fn new(timeout: Duration) -> Self {
        Self {
            disconnect_timeout: timeout,

            session_instances: HashMap::new(),
            world_instances: HashMap::new(),
            asset_instance: None,
            social_instance: None,

            assetless_session_instances: HashSet::new(),
            socialless_session_instances: HashSet::new(),
        }
    }

    pub fn register_session_instance(&mut self, instance: SessionInstance) {
        let key = (instance.http_addr().to_string(), instance.http_port());

        self.assetless_session_instances.insert(key.clone());
        self.socialless_session_instances.insert(key.clone());

        if self.session_instances.contains_key(&key) {
            info!("session instance restart detected. received re-registration request. details: {:?}", key);
            return;
        }

        self.session_instances.insert(key.clone(), instance);
    }

    pub fn deregister_session_instance(&mut self, http_addr: &str, http_port: u16) {
        let key = (http_addr.to_string(), http_port);
        self.session_instances.remove(&key);
        self.assetless_session_instances.remove(&key);
        self.socialless_session_instances.remove(&key);
    }

    pub async fn deregister_session_instance_sendreqs(&mut self, session_http_addr: &str, session_http_port: u16) {

        // send disconnect social server message to session instance
        let Some(social_instance) = &self.social_instance else {
            return;
        };
        let social_instance_addr = social_instance.http_addr().to_string();
        let social_instance_port = social_instance.http_port();
        let social_last_heard = social_instance.last_heard();

        let session_http_addr = session_http_addr.to_string();

        Server::spawn(async move {
            let request = DisconnectSessionServerRequest::new(REGION_SERVER_SECRET, &session_http_addr, session_http_port);
            let response = HttpClient::send(&social_instance_addr, social_instance_port, request).await;
            match response {
                Ok(_) => {
                    info!(
                        "from {:?}:{} - social server disconnect session server success",
                        social_instance_addr, social_instance_port
                    );
                    let mut last_heard = social_last_heard.write().await;
                    *last_heard = Instant::now();
                }
                Err(err) => {
                    warn!(
                        "from {:?}:{} - social server disconnect session server failure: {}",
                        social_instance_addr,
                        social_instance_port,
                        err.to_string()
                    );
                }
            }
        });
    }

    pub async fn deregister_asset_instance(&mut self) {
        self.asset_instance = None;

        for (key, instance) in self.session_instances.iter() {
            if !self.assetless_session_instances.contains(key) {
                self.assetless_session_instances.insert(key.clone());

                // send disconnect asset server message to session instance

                let instance_addr = instance.http_addr().to_string();
                let instance_port = instance.http_port();
                let last_heard = instance.last_heard();

                Server::spawn(async move {
                    let request = DisconnectAssetServerRequest::new(REGION_SERVER_SECRET);
                    let response = HttpClient::send(&instance_addr, instance_port, request).await;
                    match response {
                        Ok(_) => {
                            info!(
                                "from {:?}:{} - session disconnect asset server success",
                                instance_addr, instance_port
                            );
                            let mut last_heard = last_heard.write().await;
                            *last_heard = Instant::now();
                        }
                        Err(err) => {
                            warn!(
                                "from {:?}:{} - session disconnect asset server failure: {}",
                                instance_addr,
                                instance_port,
                                err.to_string()
                            );
                        }
                    }
                });
            }
        }
    }

    pub async fn deregister_social_instance(&mut self) {
        self.social_instance = None;

        for (key, instance) in self.session_instances.iter() {
            if !self.socialless_session_instances.contains(key) {
                self.socialless_session_instances.insert(key.clone());

                // send disconnect social server message to session instance

                let instance_addr = instance.http_addr().to_string();
                let instance_port = instance.http_port();
                let last_heard = instance.last_heard();

                Server::spawn(async move {
                    let request = DisconnectSocialServerRequest::new(REGION_SERVER_SECRET);
                    let response = HttpClient::send(&instance_addr, instance_port, request).await;
                    match response {
                        Ok(_) => {
                            info!(
                                "from {:?}:{} - session disconnect social server success",
                                instance_addr, instance_port
                            );
                            let mut last_heard = last_heard.write().await;
                            *last_heard = Instant::now();
                        }
                        Err(err) => {
                            warn!(
                                "from {:?}:{} - session disconnect social server failure: {}",
                                instance_addr,
                                instance_port,
                                err.to_string()
                            );
                        }
                    }
                });
            }
        }
    }

    pub fn get_available_session_server(&self) -> Option<&SessionInstance> {
        // TODO: load balance
        self.session_instances.values().next()
    }

    pub fn get_session_server_from_instance_secret(
        &self,
        instance_secret: &str,
    ) -> Option<&SessionInstance> {
        self.session_instances
            .values()
            .find(|instance| instance.instance_secret() == instance_secret)
    }

    pub fn register_world_instance(&mut self, instance: WorldInstance) {
        let key = (instance.http_addr().to_string(), instance.http_port());

        if self.world_instances.contains_key(&key) {
            info!(
                "world instance restart detected. received re-registration request. details: {:?}",
                key
            );
            return;
        }

        self.world_instances.insert(key, instance);
    }

    pub fn get_available_world_server(&self) -> Option<&WorldInstance> {
        // TODO: load balance
        self.world_instances.values().next()
    }

    pub fn register_asset_instance(&mut self, instance: AssetInstance) {
        if let Some(old_asset_instance) = self.asset_instance.as_ref() {
            if old_asset_instance.http_addr() == instance.http_addr()
                && old_asset_instance.http_port() == instance.http_port()
            {
                info!("asset instance restart detected. received re-registration request. details: {:?}{:?}", old_asset_instance.http_addr(), old_asset_instance.http_port());
            }
            panic!("shouldn't have more than one asset instance");
        } else {
            self.asset_instance = Some(instance);
        }
    }

    pub fn register_social_instance(&mut self, instance: SocialInstance) {
        if let Some(old_social_instance) = self.social_instance.as_ref() {
            if old_social_instance.http_addr() == instance.http_addr()
                && old_social_instance.http_port() == instance.http_port()
            {
                info!("social instance restart detected. received re-registration request. details: {:?}{:?}", old_social_instance.http_addr(), old_social_instance.http_port());
            }
            panic!("shouldn't have more than one social instance");
        } else {
            self.social_instance = Some(instance);
        }
    }

    pub async fn send_heartbeats(&mut self) {
        let now = Instant::now();

        self.disconnect_unheard_instances(now).await;

        // send out heartbeats
        for instance in self.session_instances.values() {
            let instance_addr = instance.http_addr().to_string();
            let instance_port = instance.http_port();
            let last_heard = instance.last_heard();

            Server::spawn(async move {
                let request = SessionHeartbeatRequest::new(REGION_SERVER_SECRET);
                let options = RequestOptions {
                    timeout_opt: Some(Duration::from_secs(1)),
                };
                let response =
                    HttpClient::send_with_options(&instance_addr, instance_port, request, options)
                        .await;
                match response {
                    Ok(_) => {
                        info!(
                            "from {:?}:{} - session heartbeat success",
                            instance_addr, instance_port
                        );
                        let mut last_heard = last_heard.write().await;
                        *last_heard = Instant::now();
                    }
                    Err(err) => {
                        warn!(
                            "from {:?}:{} - session heartbeat failure: {}",
                            instance_addr,
                            instance_port,
                            err.to_string()
                        );
                    }
                }
            });
        }

        for instance in self.world_instances.values() {
            let instance_addr = instance.http_addr().to_string();
            let instance_port = instance.http_port();

            let last_heard = instance.last_heard();

            Server::spawn(async move {
                let request = WorldHeartbeatRequest::new(REGION_SERVER_SECRET);
                let options = RequestOptions {
                    timeout_opt: Some(Duration::from_secs(1)),
                };
                let response =
                    HttpClient::send_with_options(&instance_addr, instance_port, request, options)
                        .await;
                match response {
                    Ok(_) => {
                        info!(
                            "from {:?}:{} - world heartbeat success",
                            instance_addr, instance_port
                        );
                        let mut last_heard = last_heard.write().await;
                        *last_heard = Instant::now();
                    }
                    Err(err) => {
                        warn!(
                            "from {:?}:{} - world heartbeat failure: {}",
                            instance_addr,
                            instance_port,
                            err.to_string()
                        );
                    }
                }
            });
        }

        if let Some(instance) = self.asset_instance.as_ref() {
            let instance_addr = instance.http_addr().to_string();
            let instance_port = instance.http_port();
            let last_heard = instance.last_heard();

            Server::spawn(async move {
                let request = AssetHeartbeatRequest::new(REGION_SERVER_SECRET);
                let options = RequestOptions {
                    timeout_opt: Some(Duration::from_secs(1)),
                };
                let response =
                    HttpClient::send_with_options(&instance_addr, instance_port, request, options)
                        .await;
                match response {
                    Ok(_) => {
                        info!(
                            "from {:?}:{} - asset heartbeat success",
                            instance_addr, instance_port
                        );
                        let mut last_heard = last_heard.write().await;
                        *last_heard = Instant::now();
                    }
                    Err(err) => {
                        warn!(
                            "from {:?}:{} - asset heartbeat failure: {}",
                            instance_addr,
                            instance_port,
                            err.to_string()
                        );
                    }
                }
            });
        }

        if let Some(instance) = self.social_instance.as_ref() {
            let instance_addr = instance.http_addr().to_string();
            let instance_port = instance.http_port();
            let last_heard = instance.last_heard();

            Server::spawn(async move {
                let request = SocialHeartbeatRequest::new(REGION_SERVER_SECRET);
                let options = RequestOptions {
                    timeout_opt: Some(Duration::from_secs(1)),
                };
                let response =
                    HttpClient::send_with_options(&instance_addr, instance_port, request, options)
                        .await;
                match response {
                    Ok(_) => {
                        info!(
                            "from {:?}:{} - social heartbeat success",
                            instance_addr, instance_port
                        );
                        let mut last_heard = last_heard.write().await;
                        *last_heard = Instant::now();
                    }
                    Err(err) => {
                        warn!(
                            "from {:?}:{} - social heartbeat failure: {}",
                            instance_addr,
                            instance_port,
                            err.to_string()
                        );
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

        for key in self.assetless_session_instances.iter() {
            let instance = self.session_instances.get(key).unwrap();
            let instance_addr = instance.http_addr().to_string();
            let instance_port = instance.http_port();
            let last_heard = instance.last_heard();
            let asset_server_addr = self
                .asset_instance
                .as_ref()
                .unwrap()
                .http_addr()
                .to_string();
            let asset_server_port = self.asset_instance.as_ref().unwrap().http_port();

            Server::spawn(async move {
                let request = ConnectAssetServerRequest::new(
                    REGION_SERVER_SECRET,
                    &asset_server_addr,
                    asset_server_port,
                );
                let response = HttpClient::send(&instance_addr, instance_port, request).await;
                match response {
                    Ok(_) => {
                        info!(
                            "from {:?}:{} - session connect asset server success",
                            instance_addr, instance_port
                        );
                        let mut last_heard = last_heard.write().await;
                        *last_heard = Instant::now();
                    }
                    Err(err) => {
                        warn!(
                            "from {:?}:{} - session connect asset server failure: {}",
                            instance_addr,
                            instance_port,
                            err.to_string()
                        );
                    }
                }
            });
        }

        self.assetless_session_instances.clear();
    }

    pub async fn sync_social_session_instances(&mut self) {
        if self.social_instance.is_none() {
            return;
        }
        if self.socialless_session_instances.is_empty() {
            return;
        }

        for key in self.socialless_session_instances.iter() {
            let session_instance = self.session_instances.get(key).unwrap();
            let session_instance_addr_1 = session_instance.http_addr().to_string();
            let session_instance_addr_2 = session_instance_addr_1.clone();
            let session_instance_port = session_instance.http_port();
            let session_last_heard = session_instance.last_heard();

            let social_server_addr_1 = self
                .social_instance
                .as_ref()
                .unwrap()
                .http_addr()
                .to_string();
            let social_server_addr_2 = social_server_addr_1.clone();
            let social_server_port = self.social_instance.as_ref().unwrap().http_port();
            let social_last_heard = self.social_instance.as_ref().unwrap().last_heard();

            // session server receives connection to social server
            Server::spawn(async move {
                let request = ConnectSocialServerRequest::new(
                    REGION_SERVER_SECRET,
                    &social_server_addr_1,
                    social_server_port,
                );
                let response = HttpClient::send(&session_instance_addr_1, session_instance_port, request).await;
                match response {
                    Ok(_) => {
                        info!(
                            "from {:?}:{} - session server connect social server success",
                            session_instance_addr_1, session_instance_port
                        );
                        let mut last_heard = session_last_heard.write().await;
                        *last_heard = Instant::now();
                    }
                    Err(err) => {
                        warn!(
                            "from {:?}:{} - session server connect social server failure: {}",
                            session_instance_addr_1,
                            session_instance_port,
                            err.to_string()
                        );
                    }
                }
            });

            // social server receives connection to session server
            Server::spawn(async move {
                let request = ConnectSessionServerRequest::new(
                    REGION_SERVER_SECRET,
                    &session_instance_addr_2,
                    session_instance_port,
                );
                let response = HttpClient::send(&social_server_addr_2, social_server_port, request).await;
                match response {
                    Ok(_) => {
                        info!(
                            "from {:?}:{} - social server connect session server success",
                            social_server_addr_2, social_server_port
                        );
                        let mut last_heard = social_last_heard.write().await;
                        *last_heard = Instant::now();
                    }
                    Err(err) => {
                        warn!(
                            "from {:?}:{} - social server connect session server failure: {}",
                            social_server_addr_2,
                            social_server_port,
                            err.to_string()
                        );
                    }
                }
            });
        }

        self.socialless_session_instances.clear();
    }

    async fn disconnect_unheard_instances(&mut self, now: Instant) {
        // clean up instances that have disconnected

        let timeout = self.disconnect_timeout;

        // disconnect session instances
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
                info!(
                    "session instance {:?}:{:?} disconnected",
                    addr_ip, addr_port
                );
                self.deregister_session_instance(&addr_ip, addr_port);
                self.deregister_session_instance_sendreqs(&addr_ip, addr_port).await;
            }
        }

        // disconnect world instances
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

        // disconnect asset instance
        {
            if let Some(instance) = &self.asset_instance {
                let last_heard = *instance.last_heard().read().await;
                let elapsed = now.duration_since(last_heard);
                if elapsed.as_secs() > timeout.as_secs() {
                    info!("asset instance disconnected");
                    self.deregister_asset_instance().await;
                }
            }
        }

        // disconnect asset instance
        {
            if let Some(instance) = &self.social_instance {
                let last_heard = *instance.last_heard().read().await;
                let elapsed = now.duration_since(last_heard);
                if elapsed.as_secs() > timeout.as_secs() {
                    info!("social instance disconnected");
                    self.deregister_social_instance().await;
                }
            }
        }
    }
}
