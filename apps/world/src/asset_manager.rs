use std::collections::HashMap;

use bevy_ecs::system::ResMut;
use bevy_ecs::{
    entity::Entity,
    prelude::Resource,
    system::{Commands, EntityCommands},
};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use asset_id::AssetId;
use bevy_http_client::{HttpClient, ResponseKey};

use session_server_http_proto::{UserAssetIdRequest, UserAssetIdResponse};
use world_server_naia_proto::components::{AssetEntry, AssetRef};

use crate::global::Global;

// AssetCatalog
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum AssetCatalog {
    Cube,
    HumanModel,
}

impl Into<AssetId> for AssetCatalog {
    fn into(self) -> AssetId {
        match self {
            AssetCatalog::Cube => AssetId::from_str("d51ndn").unwrap(),
            AssetCatalog::HumanModel => AssetId::from_str("dx8pye").unwrap()
        }
    }
}

// AssetData
struct AssetData {
    pub(crate) entry_entity: Entity,
}

impl AssetData {
    fn new(entry_entity: Entity) -> Self {
        Self { entry_entity }
    }
}

// UserAssetData
struct UserAssetData {
    room_key: RoomKey,
    asset_ref_counts: HashMap<AssetId, u16>,
}

impl UserAssetData {
    fn new(room_key: RoomKey) -> Self {
        Self {
            room_key,
            asset_ref_counts: HashMap::new(),
        }
    }

    // returns true if asset ref was added for the first time
    pub(crate) fn add_asset_ref(&mut self, asset_id: AssetId) -> bool {
        let count = self.asset_ref_counts.entry(asset_id).or_insert(0);
        *count += 1;

        let output = *count == 1;

        output
    }

    // returns true if asset ref was removed for the last time
    pub(crate) fn remove_asset_ref(&mut self, asset_id: AssetId) -> bool {
        let count = self.asset_ref_counts.get_mut(&asset_id).unwrap();
        *count -= 1;
        let output = *count == 0;

        if output {
            self.asset_ref_counts.remove(&asset_id);
        }

        output
    }
}

// AssetManager
#[derive(Resource)]
pub struct AssetManager {
    user_key_to_data_map: HashMap<UserKey, UserAssetData>,
    asset_id_to_data_map: HashMap<AssetId, AssetData>,
    asset_response_keys: Vec<ResponseKey<UserAssetIdResponse>>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            user_key_to_data_map: HashMap::new(),
            asset_id_to_data_map: HashMap::new(),
            asset_response_keys: Vec::new(),
        }
    }

    pub fn register_user(&mut self, server: &mut Server, user_key: &UserKey) {
        let room_key = server.make_room().key();
        let user_data = UserAssetData::new(room_key);
        self.user_key_to_data_map.insert(*user_key, user_data);
    }

    pub fn deregister_user(&mut self, user_key: &UserKey) {
        self.user_key_to_data_map.remove(user_key);
    }

    fn create_asset_ref<M: Send + Sync + 'static>(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        asset_id: AssetId,
    ) -> AssetRef<M> {
        let mut new_ref = AssetRef::new();
        if !self.asset_id_to_data_map.contains_key(&asset_id) {
            self.init_asset_id_entity(commands, server, asset_id);
        }

        let asset_data = self.asset_id_to_data_map.get(&asset_id).unwrap();
        new_ref
            .asset_id_entity
            .set(server, &asset_data.entry_entity);
        new_ref
    }

    fn init_asset_id_entity(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        asset_id: AssetId,
    ) {
        let asset_entry_entity = commands
            .spawn_empty()
            .enable_replication(server)
            .insert(AssetEntry::new(asset_id))
            .id();
        let asset_data = AssetData::new(asset_entry_entity);
        self.asset_id_to_data_map.insert(asset_id, asset_data);
    }

    pub fn update(&mut self, http_client: &mut HttpClient) {
        // check response keys
        let mut received_response_keys = Vec::new();
        for response_key in self.asset_response_keys.iter() {
            if let Some(response) = http_client.recv(response_key) {
                received_response_keys.push(response_key.clone());
                match response {
                    Ok(_response) => {
                        info!("received user asset response");
                    }
                    Err(error) => {
                        info!("error: {}", error.to_string());
                    }
                }
            }
        }
        for response_key in received_response_keys.iter() {
            self.asset_response_keys.retain(|key| key != response_key);
        }
    }

    pub fn handle_scope_actions(
        &mut self,
        server: &mut Server,
        global: &Global,
        http_client: &mut HttpClient,
        ref_actions: Vec<(UserKey, AssetId, bool)>,
    ) {
        info!("handle_scope_actions");

        let mut asset_actions = Vec::new();

        // update ref count for each user for each added/removed asset ref
        for (user_key, asset_id, added) in ref_actions {
            let Some(user_data) = self.user_key_to_data_map.get_mut(&user_key) else {
                panic!("UserAssetData not found for user_key");
            };

            if added {
                info!(
                    "adding asset ref for user: {:?}, asset: {:?}",
                    user_key, asset_id
                );
                if user_data.add_asset_ref(asset_id) {
                    // user added asset ref for asset for the first time
                    asset_actions.push((user_key, asset_id, true));
                }
            } else {
                info!("removing asset ref for user");
                if user_data.remove_asset_ref(asset_id) {
                    // user removed last asset ref for asset
                    asset_actions.push((user_key, asset_id, false));
                }
            }
        }

        // update room scope for each user for each added/removed asset entry
        for (user_key, asset_id, added) in asset_actions {
            let user_data = self.user_key_to_data_map.get(&user_key).unwrap();
            let user_room_key = user_data.room_key;
            let asset_data = self.asset_id_to_data_map.get(&asset_id).unwrap();
            let asset_entry_entity = asset_data.entry_entity;

            let mut room = server.room_mut(&user_room_key);
            if added {
                info!(
                    "adding asset entry for user: {:?}, asset: {:?}",
                    user_key, asset_id
                );
                room.add_entity(&asset_entry_entity);
                self.notify_session_server_asset(global, http_client, &user_key, asset_id, true);
            } else {
                room.remove_entity(&asset_entry_entity);
                self.notify_session_server_asset(global, http_client, &user_key, asset_id, false);
            }
        }
    }

    fn notify_session_server_asset(
        &mut self,
        global: &Global,
        http_client: &mut HttpClient,
        user_key: &UserKey,
        asset_id: AssetId,
        added: bool,
    ) {
        let instance_secret = global.instance_secret();
        let user_id = global.get_user_id(user_key).unwrap();

        let request = UserAssetIdRequest::new(instance_secret, user_id, asset_id, added);

        let (session_server_addr, session_server_port) =
            global.get_user_session_server(user_key).unwrap();
        let key = http_client.send(&session_server_addr, session_server_port, request);

        self.set_user_asset_response_key(key);
    }

    fn set_user_asset_response_key(&mut self, response_key: ResponseKey<UserAssetIdResponse>) {
        self.asset_response_keys.push(response_key);
    }
}

pub fn update(mut asset_manager: ResMut<AssetManager>, mut http_client: ResMut<HttpClient>) {
    asset_manager.update(&mut http_client);
}

// AssetCommandsExt
pub trait AssetCommandsExt {
    fn insert_asset<M: Send + Sync + 'static>(
        &mut self,
        asset_manager: &mut AssetManager,
        server: &mut Server,
        asset_id: AssetId,
    ) -> &mut Self;
}

impl AssetCommandsExt for EntityCommands<'_> {
    fn insert_asset<M: Send + Sync + 'static>(
        &mut self,
        asset_manager: &mut AssetManager,
        server: &mut Server,
        asset_id: AssetId,
    ) -> &mut Self {
        let new_ref = asset_manager.create_asset_ref::<M>(&mut self.commands(), server, asset_id);
        self.insert(new_ref);
        self
    }
}
