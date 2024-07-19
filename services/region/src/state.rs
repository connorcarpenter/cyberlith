use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use logging::info;

use crate::{
    asset_instance::AssetInstance,
    requests::{
        send_asset_heartbeat_request, send_disconnect_session_server_message_to_social_instance,
        send_disconnect_social_server_message_to_session_instance, send_session_heartbeat_request,
        send_world_heartbeat_request,
    },
    requests::{
        send_connect_asset_server_req_to_session_server,
        send_connect_session_server_req_to_social_server,
        send_connect_social_server_req_to_session_server,
        send_disconnect_asset_instance_to_session_instance, send_social_heartbeat_request,
    },
    session_instance::SessionInstance,
    social_instance::SocialInstance,
    world_instance::WorldInstance,
};

pub struct State {
    disconnect_timeout: Duration,

    session_instances: HashMap<String, SessionInstance>,
    session_url_to_instance_map: HashMap<(String, u16), String>,

    world_instances: HashMap<(String, u16), WorldInstance>,
    asset_instance: Option<AssetInstance>,
    social_instance: Option<SocialInstance>,
}

impl State {
    pub fn new(disconnect_timeout: Duration) -> Self {
        Self {
            disconnect_timeout,

            session_instances: HashMap::new(),
            session_url_to_instance_map: HashMap::new(),

            world_instances: HashMap::new(),
            asset_instance: None,
            social_instance: None,
        }
    }

    pub async fn register_session_instance(
        &mut self,
        instance_secret: &str,
        http_addr: &str,
        http_port: u16,
    ) {
        let session_instance = SessionInstance::new(instance_secret, http_addr, http_port);

        let key = session_instance.key();

        if self.session_url_to_instance_map.contains_key(&key) {
            info!("session instance restart detected. received re-registration request. details: {:?}", key);

            let old_instance_secret = self.session_url_to_instance_map.remove(&key).unwrap();
            self.deregister_session_instance(&old_instance_secret).await;
        }

        self.session_instances.insert(instance_secret.to_string(), session_instance);
        self.session_url_to_instance_map.insert(key, instance_secret.to_string());
    }

    pub async fn register_world_instance(
        &mut self,
        instance_secret: &str,
        http_addr: &str,
        http_port: u16,
    ) {
        let world_instance = WorldInstance::new(instance_secret, http_addr, http_port);

        let key = world_instance.key();

        if self.world_instances.contains_key(&key) {
            info!(
                "world instance restart detected. received re-registration request. details: {:?}",
                key
            );

            self.deregister_world_instance(http_addr, http_port).await;
        }

        self.world_instances.insert(key, world_instance);
    }

    pub async fn register_asset_instance(&mut self, http_addr: &str, http_port: u16) {
        if let Some(old_asset_instance) = self.asset_instance.as_ref() {
            if old_asset_instance.http_addr() == http_addr
                && old_asset_instance.http_port() == http_port
            {
                info!("asset instance restart detected. received re-registration request. details: {:?}{:?}", old_asset_instance.http_addr(), old_asset_instance.http_port());

                self.deregister_asset_instance().await;
            } else {
                panic!("shouldn't have more than one asset instance");
            }
        }

        let asset_instance = AssetInstance::new(http_addr, http_port);
        self.asset_instance = Some(asset_instance);
    }

    pub async fn register_social_instance(&mut self, social_addr: &str, social_port: u16) {
        if let Some(old_social_instance) = self.social_instance.as_ref() {
            if old_social_instance.http_addr() == social_addr
                && old_social_instance.http_port() == social_port
            {
                info!(
                    "social instance restart detected. received re-registration request. details: {:?}{:?}",
                    old_social_instance.http_addr(),
                    old_social_instance.http_port()
                );
                self.deregister_social_instance().await;
            } else {
                panic!("shouldn't have more than one social instance");
            }
        }

        let instance = SocialInstance::new(social_addr, social_port);
        self.social_instance = Some(instance);
    }

    pub async fn deregister_session_instance(&mut self, old_instance_secret: &str) {

        self.session_instances.remove(old_instance_secret);

        if let Some(social_instance) = self.social_instance.as_mut() {

            if social_instance.has_connected_session_server(old_instance_secret) {
                // tell social server to disconnect old session server instance
                send_disconnect_session_server_message_to_social_instance(
                    &old_instance_secret,
                    social_instance,
                )
                .await;
                social_instance.remove_connected_session_server(old_instance_secret);
            }
        }

        // TODO: tell world instances to disconnect session server instance
    }

    pub async fn deregister_world_instance(&mut self, http_addr: &str, http_port: u16) {
        let key = (http_addr.to_string(), http_port);

        self.world_instances.remove(&key);

        // TODO: disconnect world instance from any session servers
    }

    pub async fn deregister_asset_instance(&mut self) {
        self.asset_instance = None;

        for (_, session_instance) in self.session_instances.iter_mut() {
            if session_instance.has_asset_server() {
                send_disconnect_asset_instance_to_session_instance(session_instance).await;

                session_instance.clear_asset_server();
            }
        }
    }

    pub async fn deregister_social_instance(&mut self) {
        self.social_instance = None;

        // tell session servers to disconnect social server instance
        for (_, session_instance) in &mut self.session_instances {
            if session_instance.has_social_server() {
                send_disconnect_social_server_message_to_session_instance(&session_instance).await;

                session_instance.clear_social_server();
            }
        }
    }

    pub fn get_social_server(&self) -> Option<&SocialInstance> {
        self.social_instance.as_ref()
    }

    pub fn get_available_session_server(&self) -> Option<&SessionInstance> {
        // TODO: load balance
        self.session_instances.values().next()
    }

    pub fn get_session_server_from_instance_secret(
        &self,
        instance_secret: &str,
    ) -> Option<&SessionInstance> {
        self.session_instances.get(instance_secret)
    }

    pub fn get_available_world_server(&self) -> Option<&WorldInstance> {
        // TODO: load balance
        self.world_instances.values().next()
    }

    pub async fn send_heartbeats(&mut self) {
        let now = Instant::now();

        self.disconnect_unheard_instances(now).await;

        // send out heartbeats
        for instance in self.session_instances.values() {
            send_session_heartbeat_request(instance).await;
        }

        for instance in self.world_instances.values() {
            send_world_heartbeat_request(instance).await;
        }

        if let Some(instance) = self.asset_instance.as_ref() {
            send_asset_heartbeat_request(instance).await;
        }

        if let Some(instance) = self.social_instance.as_ref() {
            send_social_heartbeat_request(instance).await;
        }
    }

    pub async fn sync_asset_session_instances(&mut self) {
        let Some(asset_instance) = self.asset_instance.as_ref() else {
            return;
        };

        for (_, session_instance) in self.session_instances.iter_mut() {
            if session_instance.has_asset_server() {
                continue;
            }

            send_connect_asset_server_req_to_session_server(&asset_instance, session_instance)
                .await;

            session_instance.set_has_asset_server();
        }
    }

    pub async fn sync_social_session_instances(&mut self) {
        let Some(social_instance) = self.social_instance.as_mut() else {
            return;
        };

        for (_, session_instance) in self.session_instances.iter_mut() {
            if session_instance.has_social_server() {
                continue;
            }

            // session server receives connection to social server
            send_connect_social_server_req_to_session_server(&social_instance, session_instance)
                .await;

            session_instance.set_has_social_server();

            // social server receives connection to session server
            send_connect_session_server_req_to_social_server(&session_instance, social_instance)
                .await;

            social_instance.insert_connected_session_server(session_instance.instance_secret());
        }
    }

    async fn disconnect_unheard_instances(&mut self, now: Instant) {
        // clean up instances that have disconnected

        let timeout = self.disconnect_timeout;

        // disconnect session instances
        {
            let mut disconnected_instances = Vec::new();
            for (instance_secret, instance) in self.session_instances.iter() {
                let last_heard = *instance.last_heard().read().await;
                let elapsed = now.duration_since(last_heard);
                if elapsed.as_secs() > timeout.as_secs() {
                    disconnected_instances.push(instance_secret.clone());
                }
            }
            for instance_secret in disconnected_instances {
                info!("session instance {:?} disconnected", instance_secret);
                self.deregister_session_instance(&instance_secret).await;
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
            for (http_addr, http_port) in disconnected_instances {
                info!(
                    "world instance {:?}:{:?} disconnected",
                    http_addr, http_port
                );
                self.deregister_world_instance(&http_addr, http_port).await;
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

        // disconnect social instance
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
