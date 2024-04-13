use std::collections::HashMap;

use bevy_ecs::{
    prelude::Resource,
    system::{Res, ResMut},
};
use logging::info;

use naia_bevy_server::{Server, UserKey};

use asset_id::AssetId;
use bevy_http_client::HttpClient;

use crate::{
    asset::{asset_store::AssetStore, user_assets::UserAssets},
    global::Global,
};

#[derive(Resource)]
pub struct AssetManager {
    users: HashMap<UserKey, UserAssets>,
    asset_store: AssetStore,
    queued_user_asset_requests: Vec<(UserKey, AssetId, bool)>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            asset_store: AssetStore::new(),
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
        naia_server: &mut Server,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
    ) {
        for (user_key, asset_id, added) in std::mem::take(&mut self.queued_user_asset_requests) {
            info!("processing queued user asset request..");
            self.handle_user_asset_request(
                naia_server,
                http_client,
                asset_server_addr,
                asset_server_port,
                user_key,
                &asset_id,
                added,
            );
        }
    }

    pub fn handle_user_asset_request(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
        user_key: UserKey,
        asset_id: &AssetId,
        added: bool,
    ) {
        if let Some(user_assets) = self.users.get_mut(&user_key) {
            user_assets.handle_user_asset_request(
                server,
                http_client,
                asset_server_addr,
                asset_server_port,
                &self.asset_store,
                asset_id,
                added,
            );
        }
    }

    pub fn process_in_flight_requests(
        &mut self,
        server: &mut Server,
        http_client: &mut HttpClient,
        asset_server_addr: &str,
        asset_server_port: u16,
    ) {
        for user_assets in self.users.values_mut() {
            user_assets.process_in_flight_requests(
                server,
                http_client,
                asset_server_addr,
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
    global: Res<Global>,
) {
    if let Some((asset_server_addr, asset_server_port)) = global.get_asset_server_url() {
        asset_manager.process_in_flight_requests(
            &mut server,
            &mut http_client,
            &asset_server_addr,
            asset_server_port,
        );
    } else {
        // it's okay to wait until the asset server is available
    }
}
