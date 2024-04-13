use std::{any::TypeId, collections::HashMap};

use bevy_ecs::{
    change_detection::{Mut, ResMut},
    entity::Entity,
    event::{EventReader, Events},
    prelude::{Query, Resource},
    system::SystemState,
    world::World,
};
use logging::info;

use world_server_naia_proto::components::{Alt1, AssetEntry, AssetRef, Main};

use asset_id::AssetId;
use asset_loader::{AssetMetadataStore, TypedAssetId};

use crate::{
    asset_cache::{AssetCache, AssetLoadedEvent},
    world::WorldClient,
};

use super::world_events::InsertAssetRefEvent;

type AssetProcessorId = TypeId;

/// Stores asset data in RAM
#[derive(Resource)]
pub struct AssetRefProcessor {
    asset_processor_map: HashMap<AssetProcessorId, Box<dyn AssetDeferredProcessor>>,
    // entry entity -> (ref entity -> asset processor id)
    entry_waitlist: HashMap<Entity, HashMap<Entity, AssetProcessorId>>,
    // asset id -> (ref entity -> asset processor id)
    ref_waitlist: HashMap<AssetId, HashMap<Entity, AssetProcessorId>>,
}

impl Default for AssetRefProcessor {
    fn default() -> Self {
        Self {
            asset_processor_map: HashMap::new(),
            entry_waitlist: HashMap::new(),
            ref_waitlist: HashMap::new(),
        }
    }
}

impl AssetRefProcessor {
    pub fn get_asset_processor_ref(
        &self,
        asset_processor_id: &AssetProcessorId,
    ) -> Option<&Box<dyn AssetDeferredProcessor>> {
        self.asset_processor_map.get(asset_processor_id)
    }

    // used by systems
    pub fn insert_asset_ref_events<T: AssetProcessor>(
        client: &WorldClient,
        asset_cache: &AssetCache,
        metadata_store: &mut AssetMetadataStore,
        asset_ref_processor: &mut AssetRefProcessor,
        asset_entry_q: &Query<&AssetEntry>,
        asset_ref_q: &Query<&AssetRef<T>>,
        entity: &Entity,
    ) -> Vec<(Entity, TypedAssetId)> {
        let Ok(asset_ref) = asset_ref_q.get(*entity) else {
            panic!("Shouldn't happen");
        };
        let Some(asset_entry_entity) = asset_ref.asset_id_entity.get(client) else {
            panic!("Shouldn't happen");
        };
        if let Ok(asset_entry) = asset_entry_q.get(asset_entry_entity) {
            let asset_id = *asset_entry.asset_id;
            let output = asset_ref_processor.handle_entity_added_asset_ref::<T>(
                asset_cache,
                metadata_store,
                entity,
                &asset_id,
            );
            return output;
        } else {
            // asset entry entity has been replicated, but not the component just yet ...
            asset_ref_processor.handle_add_asset_entry_waitlist::<T>(entity, &asset_entry_entity);
            return Vec::new();
        };
    }

    // used as a system
    pub fn handle_asset_loaded_events(world: &mut World) {
        let mut system_state: SystemState<(
            EventReader<AssetLoadedEvent>,
            ResMut<AssetRefProcessor>,
        )> = SystemState::new(world);
        let (mut reader, mut asset_ref_processer) = system_state.get_mut(world);

        let mut list_of_events = Vec::new();
        for event in reader.read() {
            info!(
                "received Asset Loaded Event! (asset_id: {:?}, asset_type: {:?})",
                event.asset_id, event.asset_type
            );

            let asset_id = event.asset_id;
            let asset_type = event.asset_type;

            // process any refs waiting for this asset
            if let Some(ref_waitlist_entry) = asset_ref_processer.ref_waitlist.remove(&asset_id) {
                let typed_asset_id = TypedAssetId::new(asset_id, asset_type);

                for (entity, asset_processor_id) in ref_waitlist_entry {
                    list_of_events.push((asset_processor_id, entity, typed_asset_id));
                }
            }
        }

        world.resource_scope(|world, asset_ref_processor: Mut<AssetRefProcessor>| {
            for (asset_processor_id, entity, typed_asset_id) in list_of_events {
                let asset_processor = asset_ref_processor
                    .get_asset_processor_ref(&asset_processor_id)
                    .unwrap();
                asset_processor.deferred_process(world, &entity, &typed_asset_id);
            }
        });
    }

    // entry entity is here, just not the component just yet ...
    pub fn handle_add_asset_entry_waitlist<T: AssetProcessor>(
        &mut self,
        ref_entity: &Entity,
        entry_entity: &Entity,
    ) {
        info!(
            "entity ({:?}) received AssetRef from World Server! waiting on asset entry..",
            ref_entity
        );
        // initialize asset processor if needed
        let asset_processor_id = T::id();
        if !self.asset_processor_map.contains_key(&asset_processor_id) {
            self.asset_processor_map
                .insert(asset_processor_id, T::make_deferred_box());
        }

        if !self.entry_waitlist.contains_key(entry_entity) {
            self.entry_waitlist.insert(*entry_entity, HashMap::new());
        }
        let entry_waitlist_entry = self.entry_waitlist.get_mut(entry_entity).unwrap();
        entry_waitlist_entry.insert(*ref_entity, asset_processor_id);
    }

    pub fn handle_add_asset_entry(
        &mut self,
        metadata_store: &mut AssetMetadataStore,
        asset_cache: &AssetCache,
        entry_entity: &Entity,
        asset_id: &AssetId,
    ) -> Vec<(AssetProcessorId, Entity, TypedAssetId)> {
        let mut output = Vec::new();
        info!(
            "entity ({:?}) received AssetEntry from World Server! (asset_id: {:?})",
            entry_entity, asset_id
        );
        // initialize asset processor if needed
        if let Some(waitlist_entry) = self.entry_waitlist.remove(entry_entity) {
            for (ref_entity, asset_processor_id) in waitlist_entry {
                info!("Processing waiting AssetRef: {:?}", ref_entity);

                // mirror logic in handle_entity_added_asset_ref
                if asset_cache.has_asset(asset_id) {
                    // process asset ref
                    let asset_type = metadata_store.get(asset_id).unwrap().asset_type();
                    let typed_asset_id = TypedAssetId::new(*asset_id, asset_type);
                    output.push((asset_processor_id, ref_entity, typed_asset_id));
                } else {
                    // put ref into waitlist
                    info!("asset {:?} not yet loaded. adding to waitlist", asset_id);
                    if !self.ref_waitlist.contains_key(asset_id) {
                        self.ref_waitlist.insert(*asset_id, HashMap::new());
                    }
                    let ref_waitlist_entry = self.ref_waitlist.get_mut(asset_id).unwrap();
                    ref_waitlist_entry.insert(ref_entity, asset_processor_id);
                }
            }
        }

        output
    }

    pub fn handle_entity_added_asset_ref<T: AssetProcessor>(
        &mut self,
        asset_cache: &AssetCache,
        metadata_store: &mut AssetMetadataStore,
        ref_entity: &Entity,
        asset_id: &AssetId,
    ) -> Vec<(Entity, TypedAssetId)> {
        info!(
            "entity ({:?}) received AssetRef from World Server! (asset_id: {:?})",
            ref_entity, asset_id
        );
        let mut output = Vec::new();
        if asset_cache.has_asset(asset_id) {
            // process asset ref
            let asset_type = metadata_store.get(asset_id).unwrap().asset_type();
            let typed_asset_id = TypedAssetId::new(*asset_id, asset_type);
            output.push((*ref_entity, typed_asset_id));
        } else {
            // initialize asset processor if needed
            let asset_processor_id = T::id();
            if !self.asset_processor_map.contains_key(&asset_processor_id) {
                self.asset_processor_map
                    .insert(asset_processor_id, T::make_deferred_box());
            }

            // put ref into waitlist
            info!("asset {:?} not yet loaded. adding to waitlist", asset_id);
            if !self.ref_waitlist.contains_key(asset_id) {
                self.ref_waitlist.insert(*asset_id, HashMap::new());
            }
            let ref_waitlist_entry = self.ref_waitlist.get_mut(asset_id).unwrap();
            ref_waitlist_entry.insert(*ref_entity, asset_processor_id);
        }
        output
    }
}

pub trait AssetProcessor: Send + Sync + 'static {
    fn id() -> AssetProcessorId;
    fn make_deferred_box() -> Box<dyn AssetDeferredProcessor>;
    fn process(world: &mut World, entity: &Entity, typed_asset_id: &TypedAssetId);
}

pub trait AssetDeferredProcessor: Send + Sync + 'static {
    fn deferred_process(&self, world: &mut World, entity: &Entity, typed_asset_id: &TypedAssetId);
}

impl AssetProcessor for Main {
    fn id() -> AssetProcessorId {
        TypeId::of::<Self>()
    }
    fn make_deferred_box() -> Box<dyn AssetDeferredProcessor> {
        Box::new(Self)
    }
    fn process(world: &mut World, entity: &Entity, typed_asset_id: &TypedAssetId) {
        let mut event_writer = world
            .get_resource_mut::<Events<InsertAssetRefEvent<Main>>>()
            .unwrap();
        event_writer.send(InsertAssetRefEvent::<Self>::new(
            *entity,
            typed_asset_id.get_id(),
            typed_asset_id.get_type(),
        ));
    }
}

impl AssetDeferredProcessor for Main {
    fn deferred_process(&self, world: &mut World, entity: &Entity, typed_asset_id: &TypedAssetId) {
        Self::process(world, entity, typed_asset_id);
    }
}

impl AssetProcessor for Alt1 {
    fn id() -> AssetProcessorId {
        TypeId::of::<Self>()
    }
    fn make_deferred_box() -> Box<dyn AssetDeferredProcessor> {
        Box::new(Self)
    }
    fn process(world: &mut World, entity: &Entity, typed_asset_id: &TypedAssetId) {
        let mut event_writer = world
            .get_resource_mut::<Events<InsertAssetRefEvent<Alt1>>>()
            .unwrap();
        event_writer.send(InsertAssetRefEvent::<Self>::new(
            *entity,
            typed_asset_id.get_id(),
            typed_asset_id.get_type(),
        ));
    }
}

impl AssetDeferredProcessor for Alt1 {
    fn deferred_process(&self, world: &mut World, entity: &Entity, typed_asset_id: &TypedAssetId) {
        Self::process(world, entity, typed_asset_id);
    }
}
