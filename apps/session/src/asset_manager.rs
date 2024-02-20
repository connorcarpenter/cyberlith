use std::collections::HashMap;

use bevy_ecs::prelude::Resource;
use bevy_ecs::system::ResMut;
use bevy_log::info;

use naia_bevy_server::{Server, UserKey};

use asset_id::AssetId;
use bevy_http_client::HttpClient;

use crate::{asset_cache::AssetCache, user_assets::UserAssets};

#[derive(Resource)]
pub struct AssetManager {
    users: HashMap<UserKey, UserAssets>,
    asset_store: AssetCache,
    queued_user_asset_requests: Vec<(UserKey, AssetId, bool)>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            asset_store: AssetCache::new(),
            queued_user_asset_requests: Vec::new(),
        }
    }

    pub fn register_user(&mut self, user_key: &UserKey) {
        self.users.insert(*user_key, UserAssets::new(user_key));
    }

    pub fn deregister_user(&mut self, user_key: &UserKey) {
        self.users.remove(user_key);
    }

    pub fn queue_user_asset_request(&mut self, user_key: UserKey, asset_id: &AssetId, added: bool) {
        self.queued_user_asset_requests
            .push((user_key, asset_id.clone(), added));
    }

    pub fn has_queued_user_asset_requests(&self) -> bool {
        self.queued_user_asset_requests.len() > 0
    }

    pub fn process_queued_user_asset_requests(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
    ) {
        for (user_key, asset_id, added) in std::mem::take(&mut self.queued_user_asset_requests) {
            info!("processing queued user asset request..");
            self.user_asset_request(
                server,
                http_client,
                asset_server_addr,
                asset_server_port,
                user_key,
                &asset_id,
                added,
            );
        }
    }

    pub fn user_asset_request(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        user_key: UserKey,
        asset_id: &AssetId,
        added: bool,
    ) {
        let user_assets = self.users.get_mut(&user_key).unwrap();
        user_assets.user_asset_request(
            server,
            http_client,
            asset_server_addr,
            asset_server_port,
            &self.asset_store,
            asset_id,
            added,
        );
    }

    pub fn process_in_flight_requests(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
    ) -> Option<Vec<(UserKey, AssetId)>> {
        let mut pending_requests = Vec::new();
        for user_assets in self.users.values_mut() {
            let (asset_server_responses, pending_client_requests) =
                user_assets.process_in_flight_requests(server, http_client);
            if let Some(asset_server_responses) = asset_server_responses {
                for (asset_id, etag, data) in asset_server_responses {
                    self.asset_store.insert_data(asset_id, etag, data);
                }
            }
            if let Some(mut pending_client_requests) = pending_client_requests {
                pending_requests.append(&mut pending_client_requests);
            }
        }
        if !pending_requests.is_empty() {
            Some(pending_requests)
        } else {
            None
        }
    }

    pub fn send_client_asset_data(
        &mut self,
        server: &mut Server,
        user_key: &UserKey,
        asset_id: &AssetId,
    ) {
        let user_assets = self.users.get_mut(user_key).unwrap();
        user_assets.send_client_asset_data(server, &self.asset_store, asset_id);
    }
}

pub fn update(
    mut asset_manager: ResMut<AssetManager>,
    mut server: Server,
    mut http_client: ResMut<HttpClient>,
) {
    let pending_client_reqs_opt =
        asset_manager.process_in_flight_requests(&mut server, &mut http_client);
    if let Some(pending_client_reqs) = pending_client_reqs_opt {
        for (user_key, asset_id) in pending_client_reqs {
            asset_manager.send_client_asset_data(&mut server, &user_key, &asset_id);
        }
    }
}
