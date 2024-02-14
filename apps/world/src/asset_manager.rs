use std::collections::HashMap;

use bevy_ecs::{entity::Entity, prelude::Resource, system::{Commands, EntityCommands}};

use naia_bevy_server::{CommandsExt, Server};

use world_server_naia_proto::components::{AssetEntry, AssetId, AssetRef};


#[derive(Clone, Copy, Eq, PartialEq)]
pub enum AssetCatalog {
    Cube,
}

impl Into<AssetId> for AssetCatalog {
    fn into(self) -> AssetId {
        match self {
            AssetCatalog::Cube => AssetId::from_str("d51ndn").unwrap(),
        }
    }
}

#[derive(Resource)]
pub struct AssetManager {
    asset_id_map: HashMap<AssetId, Entity>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            asset_id_map: HashMap::new(),
        }
    }

    fn new_ref <
        M: Send + Sync + 'static,
    > (
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        asset_id: AssetId,
    ) -> AssetRef<M> {

        let mut new_ref = AssetRef::new();
        if !self.asset_id_map.contains_key(&asset_id) {
            self.init_asset_id_entity(commands, server, asset_id);
        }

        let asset_id_entity = self.asset_id_map.get(&asset_id).unwrap();
        new_ref.asset_id_entity.set(server, asset_id_entity);
        new_ref
    }

    fn init_asset_id_entity(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        asset_id: AssetId,
    ) {
        let asset_id_entity = commands
            .spawn_empty()
            .enable_replication(server)
            .insert(AssetEntry::new(asset_id))
            .id();
        self.asset_id_map.insert(asset_id, asset_id_entity);
    }
}

pub trait AssetCommandsExt {
    fn insert_asset<M: Send + Sync + 'static>(&mut self, asset_manager: &mut AssetManager, server: &mut Server, asset_id: AssetId) -> &mut Self;
}

impl AssetCommandsExt for EntityCommands<'_, '_, '_> {
    fn insert_asset<M: Send + Sync + 'static>(&mut self, asset_manager: &mut AssetManager, server: &mut Server, asset_id: AssetId) -> &mut Self {
        let new_ref = asset_manager.new_ref::<M>(self.commands(), server, asset_id);
        self.insert(new_ref);
        self
    }
}