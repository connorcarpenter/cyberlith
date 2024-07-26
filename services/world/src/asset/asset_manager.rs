use std::{any::TypeId, collections::HashMap};

use bevy_ecs::{
    entity::Entity,
    prelude::{Query, RemovedComponents, Resource},
    query::Added,
    system::{Commands, Res, ResMut, SystemState},
    world::World,
};

use naia_bevy_server::{CommandsExt, RoomKey, Server, UserKey};

use asset_id::AssetId;
use bevy_http_client::{HttpClient, ResponseKey};
use logging::info;

use session_server_http_proto::{UserAssetIdRequest, UserAssetIdResponse};
use world_server_naia_proto::components::{Alt1, AssetEntry, AssetRef, Main};

use crate::{user::UserManager, world_instance::WorldInstance};

// AssetCatalog
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum AssetCatalog {
    AvatarUnit,
}

impl Into<AssetId> for AssetCatalog {
    fn into(self) -> AssetId {
        match self {
            AssetCatalog::AvatarUnit => AssetId::from_str("h1g2dt").unwrap(),
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
    type_to_asset_ref_entity_map: HashMap<TypeId, HashMap<Entity, AssetId>>,
    asset_response_keys: Vec<ResponseKey<UserAssetIdResponse>>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            user_key_to_data_map: HashMap::new(),
            asset_id_to_data_map: HashMap::new(),
            type_to_asset_ref_entity_map: HashMap::new(),
            asset_response_keys: Vec::new(),
        }
    }
}

impl AssetManager {
    pub fn register_user(&mut self, server: &mut Server, user_key: &UserKey) {
        let room_key = server.make_room().key();
        server.room_mut(&room_key).add_user(user_key);
        let user_data = UserAssetData::new(room_key);
        self.user_key_to_data_map.insert(*user_key, user_data);
    }

    pub fn deregister_user(&mut self, user_key: &UserKey) {
        let data = self.user_key_to_data_map.remove(user_key).unwrap();
        let _room_key = data.room_key;
        // TODO: do we need to remove user from room?
    }

    pub(crate) fn create_asset_ref<M: Send + Sync + 'static>(
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
        let entry_entity = asset_data.entry_entity;
        new_ref.asset_id_entity.set(server, &entry_entity);
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

    pub(crate) fn handle_asset_ref_scope_events(
        world: &mut World,
        scope_actions: HashMap<(UserKey, Entity), bool>,
    ) {
        // 4. determine if any entities that have gone into or out of scope have AssetRef components
        let mut asset_id_entity_actions = Vec::new();

        Self::get_asset_id_entity_actions::<Main>(
            world,
            &scope_actions,
            &mut asset_id_entity_actions,
        );
        Self::get_asset_id_entity_actions::<Alt1>(
            world,
            &scope_actions,
            &mut asset_id_entity_actions,
        );

        if asset_id_entity_actions.is_empty() {
            return;
        }

        // update asset id entities in asset manager
        {
            let mut system_state: SystemState<(
                Server,
                Res<WorldInstance>,
                Res<UserManager>,
                ResMut<AssetManager>,
                ResMut<HttpClient>,
            )> = SystemState::new(world);
            let (mut server, world_instance, user_manager, mut asset_manager, mut http_client) =
                system_state.get_mut(world);

            asset_manager.handle_scope_actions(
                &mut server,
                &world_instance,
                &user_manager,
                &mut http_client,
                asset_id_entity_actions,
            );
        };
    }

    fn get_asset_id_entity_actions<T: Send + Sync + 'static>(
        world: &mut World,
        scope_actions: &HashMap<(UserKey, Entity), bool>,
        asset_id_entity_actions: &mut Vec<(UserKey, AssetId, bool)>,
    ) {
        let marker_type_id = TypeId::of::<T>();

        let mut system_state: SystemState<(
            Server,
            ResMut<AssetManager>,
            Query<&AssetEntry>,
            Query<&AssetRef<T>>,
        )> = SystemState::new(world);
        let (server, mut asset_manager, asset_entry_q, asset_ref_q) = system_state.get_mut(world);

        for ((user_key, entity), include) in scope_actions.iter() {
            // determine if entity has any AssetRef components
            info!("Checking entity for AssetRefs: {:?}", entity);

            // AssetRef<Main>
            if let Ok(asset_ref) = asset_ref_q.get(*entity) {
                let asset_id_entity = asset_ref.asset_id_entity.get(&server).unwrap();
                let asset_id = *asset_entry_q.get(asset_id_entity).unwrap().asset_id;

                info!("entity {:?} has AssetRef(asset_id: {:?})", entity, asset_id);

                if *include {
                    asset_manager.insert_asset_ref_entity(&marker_type_id, entity, &asset_id);
                } else {
                    asset_manager.remove_asset_ref_entity(&marker_type_id, entity);
                }
                asset_id_entity_actions.push((*user_key, asset_id, *include));
            }
        }
    }

    fn handle_scope_actions(
        &mut self,
        server: &mut Server,
        world_instance: &WorldInstance,
        user_manager: &UserManager,
        http_client: &mut HttpClient,
        ref_actions: Vec<(UserKey, AssetId, bool)>,
    ) {
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
                info!(
                    "removing asset ref for user: {:?}, asset: {:?}",
                    user_key, asset_id
                );
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
                self.notify_session_server_asset(
                    world_instance,
                    user_manager,
                    http_client,
                    &user_key,
                    asset_id,
                    true,
                );
            } else {
                info!(
                    "removing asset entry for user: {:?}, asset: {:?}",
                    user_key, asset_id
                );
                room.remove_entity(&asset_entry_entity);
                self.notify_session_server_asset(
                    world_instance,
                    user_manager,
                    http_client,
                    &user_key,
                    asset_id,
                    false,
                );
            }
        }
    }

    fn notify_session_server_asset(
        &mut self,
        world_instance: &WorldInstance,
        user_manager: &UserManager,
        http_client: &mut HttpClient,
        user_key: &UserKey,
        asset_id: AssetId,
        added: bool,
    ) {
        let instance_secret = world_instance.instance_secret();
        let user_id = user_manager.get_user_id(user_key).unwrap();

        let request = UserAssetIdRequest::new(instance_secret, user_id, asset_id, added);

        let (session_server_addr, session_server_port) =
            user_manager.get_user_session_server(user_key).unwrap();
        info!(
            "sending user asset request to session server: {:?}, {:?}, {:?}",
            session_server_addr, session_server_port, instance_secret
        );
        let key = http_client.send(&session_server_addr, session_server_port, request);

        self.set_user_asset_response_key(key);
    }

    fn set_user_asset_response_key(&mut self, response_key: ResponseKey<UserAssetIdResponse>) {
        self.asset_response_keys.push(response_key);
    }

    fn insert_asset_ref_entity(
        &mut self,
        marker_type_id: &TypeId,
        entity: &Entity,
        asset_id: &AssetId,
    ) {
        if !self
            .type_to_asset_ref_entity_map
            .contains_key(marker_type_id)
        {
            self.type_to_asset_ref_entity_map
                .insert(*marker_type_id, HashMap::new());
        }
        let asset_ref_entity_map = self
            .type_to_asset_ref_entity_map
            .get_mut(marker_type_id)
            .unwrap();
        asset_ref_entity_map.insert(*entity, *asset_id);
    }

    fn remove_asset_ref_entity(
        &mut self,
        marker_type_id: &TypeId,
        entity: &Entity,
    ) -> Option<AssetId> {
        let asset_ref_entity_map = self.type_to_asset_ref_entity_map.get_mut(marker_type_id)?;
        let result = asset_ref_entity_map.remove(entity);
        if asset_ref_entity_map.is_empty() {
            self.type_to_asset_ref_entity_map.remove(marker_type_id);
        }
        result
    }
}

// Systems

pub fn update(mut asset_manager: ResMut<AssetManager>, mut http_client: ResMut<HttpClient>) {
    asset_manager.update(&mut http_client);
}

pub fn handle_asset_ref_added_events<T: Send + Sync + 'static>(
    mut server: Server,
    mut asset_manager: ResMut<AssetManager>,
    world_instance: Res<WorldInstance>,
    user_manager: Res<UserManager>,
    mut http_client: ResMut<HttpClient>,
    asset_entry_q: Query<&AssetEntry>,
    asset_ref_main_q: Query<(Entity, &AssetRef<T>), Added<AssetRef<T>>>,
) {
    let mut entities = Vec::new();
    for (entity, _) in asset_ref_main_q.iter() {
        entities.push(entity);
    }

    if entities.is_empty() {
        return;
    }

    let marker_type_id = TypeId::of::<T>();
    let mut asset_ref_actions = Vec::new();
    let user_keys = server.user_keys();

    for entity in entities {
        for user_key in user_keys.iter() {
            if server.user_scope(&user_key).has(&entity) {
                let (_, asset_ref) = asset_ref_main_q.get(entity).unwrap();
                let asset_entry_entity = asset_ref.asset_id_entity.get(&server).unwrap();
                let asset_entry = asset_entry_q.get(asset_entry_entity).unwrap();
                let asset_id = *asset_entry.asset_id;
                asset_manager.insert_asset_ref_entity(&marker_type_id, &entity, &asset_id);
                asset_ref_actions.push((*user_key, asset_id, true));
            }
        }
    }

    asset_manager.handle_scope_actions(
        &mut server,
        &world_instance,
        &user_manager,
        &mut http_client,
        asset_ref_actions,
    );
}

pub fn handle_asset_ref_removed_events<T: Send + Sync + 'static>(
    mut server: Server,
    mut asset_manager: ResMut<AssetManager>,
    world_instance: Res<WorldInstance>,
    user_manager: Res<UserManager>,
    mut http_client: ResMut<HttpClient>,
    mut removals: RemovedComponents<AssetRef<T>>,
) {
    let mut entities = Vec::new();
    for entity in removals.read() {
        entities.push(entity);
    }

    if entities.is_empty() {
        return;
    }

    let mut asset_ref_actions = Vec::new();
    let type_id = TypeId::of::<T>();
    let user_keys = server.user_keys();

    for entity in entities {
        for user_key in user_keys.iter() {
            if server.user_scope(&user_key).has(&entity) {
                let asset_id = asset_manager
                    .remove_asset_ref_entity(&type_id, &entity)
                    .unwrap();
                asset_ref_actions.push((*user_key, asset_id, false));
            }
        }
    }

    asset_manager.handle_scope_actions(
        &mut server,
        &world_instance,
        &user_manager,
        &mut http_client,
        asset_ref_actions,
    );
}
