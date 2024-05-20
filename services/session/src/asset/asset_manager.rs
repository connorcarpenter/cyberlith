use std::collections::HashMap;

use bevy_ecs::{
    prelude::Resource,
    system::ResMut,
};
use logging::info;

use naia_bevy_server::{Server, UserKey};

use asset_id::AssetId;
use bevy_http_client::HttpClient;

use crate::asset::{asset_store::AssetStore, user_assets::UserAssets};

#[derive(Resource)]
pub struct AssetManager {
    users: HashMap<UserKey, UserAssets>,
    asset_store: AssetStore,
    queued_user_asset_requests: Vec<(UserKey, AssetId, bool)>,
    asset_server_opt: Option<(String, u16)>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            asset_store: AssetStore::new(),
            queued_user_asset_requests: Vec::new(),
            asset_server_opt: None,
        }
    }

    // Asset Server

    pub fn set_asset_server(&mut self, addr: &str, port: u16) {
        self.asset_server_opt = Some((addr.to_string(), port));
    }

    pub fn clear_asset_server(&mut self) {
        self.asset_server_opt = None;
    }

    pub fn get_asset_server_url(&self) -> Option<(String, u16)> {
        self.asset_server_opt
            .as_ref()
            .map(|(addr, port)| (addr.clone(), *port))
    }

    pub fn register_user(&mut self, user_key: &UserKey) {
        self.users.insert(*user_key, UserAssets::new(user_key));
    }

    pub fn deregister_user(&mut self, user_key: &UserKey) {
        self.users.remove(user_key);
    }

    fn queue_user_asset_request(&mut self, user_key: UserKey, asset_id: &AssetId, added: bool) {
        self.queued_user_asset_requests
            .push((user_key, asset_id.clone(), added));
    }

    pub fn has_queued_user_asset_requests(&self) -> bool {
        self.queued_user_asset_requests.len() > 0
    }

    pub fn process_queued_user_asset_requests(
        &mut self,
        naia_server: &mut Server,
        http_client: &mut HttpClient,
    ) {
        if !self.has_queued_user_asset_requests() {
            // no queued assets
            return;
        }
        if self.get_asset_server_url().is_none() {
            // it's okay to wait until the asset server is available
            return;
        };

        for (user_key, asset_id, added) in std::mem::take(&mut self.queued_user_asset_requests) {
            info!("processing queued user asset request..");
            if added {
                self.load_user_asset(
                    naia_server,
                    http_client,
                    user_key,
                    &asset_id,
                );
            } else {
                self.unload_user_asset(
                    user_key,
                    &asset_id,
                );
            }
        }
    }

    pub fn load_user_asset(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
        user_key: UserKey,
        asset_id: &AssetId,
    ) {
        if let Some((asset_server_addr, asset_server_port)) = self.get_asset_server_url() {
            if let Some(user_assets) = self.users.get_mut(&user_key) {
                user_assets.load_user_asset(
                    server,
                    http_client,
                    &asset_server_addr,
                    asset_server_port,
                    &self.asset_store,
                    asset_id,
                );
            } else {
                panic!("UserAssets not found for user_key: {:?}", user_key);
            }
        } else {
            info!("Asset Server not available, queuing request ..");
            self.queue_user_asset_request(user_key, asset_id, true);
        }
    }

    pub fn unload_user_asset(
        &mut self,
        user_key: UserKey,
        asset_id: &AssetId,
    ) {
        if let Some((_asset_server_addr, _asset_server_port)) = self.get_asset_server_url() {
            if let Some(user_assets) = self.users.get_mut(&user_key) {
                user_assets.unload_user_asset(asset_id);
            } else {
                panic!("UserAssets not found for user_key: {:?}", user_key);
            }
        } else {
            info!("Asset Server not available, queuing request ..");
            self.queue_user_asset_request(user_key, asset_id, false);
        }
    }

    pub fn process_in_flight_requests(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
    ) {
        let Some((asset_server_addr, asset_server_port)) = self.get_asset_server_url() else {
            // it's okay to wait until the asset server is available
            return;
        };

        for user_assets in self.users.values_mut() {
            user_assets.process_in_flight_requests(
                server,
                http_client,
                &asset_server_addr,
                asset_server_port,
                &mut self.asset_store,
            );
        }
    }
}

pub fn update(
    mut asset_manager: ResMut<AssetManager>,
    mut server: Server,
    mut http_client: ResMut<HttpClient>,
) {
    asset_manager.process_in_flight_requests(
        &mut server,
        &mut http_client,
    );
    asset_manager.process_queued_user_asset_requests(
        &mut server,
        &mut http_client,
    );
}
