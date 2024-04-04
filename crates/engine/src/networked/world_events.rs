use bevy_ecs::{
    change_detection::{Mut, ResMut},
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    prelude::{Query, Resource, World as BevyWorld},
    system::{Res, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{events::InsertComponentEvents, Replicate};

use world_server_naia_proto::components::{Alt1, AssetEntry, AssetRef, Main, Position};

use asset_id::{AssetId, AssetType};
use asset_loader::AssetMetadataStore;

use crate::{asset_cache::AssetCache, world::WorldClient};
use super::{
    asset_ref_processor::{AssetProcessor, AssetRefProcessor},
    client_markers::World,
};

#[derive(Event)]
pub struct InsertComponentEvent<T: Replicate> {
    pub entity: Entity,
    phantom_t: std::marker::PhantomData<T>,
}

impl<T: Replicate> InsertComponentEvent<T> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            phantom_t: std::marker::PhantomData,
        }
    }
}

#[derive(Event)]
pub struct InsertAssetRefEvent<T> {
    pub entity: Entity,
    pub asset_id: AssetId,
    pub asset_type: AssetType,

    phantom_t: std::marker::PhantomData<T>,
}

impl<T> InsertAssetRefEvent<T> {
    pub fn new(entity: Entity, asset_id: AssetId, asset_type: AssetType) -> Self {
        Self {
            entity,
            asset_id,
            asset_type,
            phantom_t: std::marker::PhantomData,
        }
    }
}

#[derive(Resource)]
struct CachedInsertComponentEventsState {
    event_state: SystemState<EventReader<'static, 'static, InsertComponentEvents<World>>>,
}

pub fn insert_component_event_startup(world: &mut BevyWorld) {
    let initial_state: SystemState<EventReader<InsertComponentEvents<World>>> =
        SystemState::new(world);
    world.insert_resource(CachedInsertComponentEventsState {
        event_state: initial_state,
    });
}

pub fn insert_component_events(world: &mut BevyWorld) {
    let mut events_collection: Vec<InsertComponentEvents<World>> = Vec::new();

    world.resource_scope(
        |world, mut events_reader_state: Mut<CachedInsertComponentEventsState>| {
            let mut events_reader = events_reader_state.event_state.get_mut(world);

            for events in events_reader.read() {
                let events_clone: InsertComponentEvents<World> = events.clone();
                events_collection.push(events_clone);
            }
        },
    );

    for events in events_collection {
        // asset events
        insert_asset_entry_event(world, &events);
        insert_asset_ref_event::<Main>(world, &events);
        insert_asset_ref_event::<Alt1>(world, &events);

        // other events
        insert_component_event::<Position>(world, &events);
    }
}

fn insert_component_event<T: Replicate>(
    world: &mut BevyWorld,
    events: &InsertComponentEvents<World>,
) {
    let mut system_state: SystemState<EventWriter<InsertComponentEvent<T>>> =
        SystemState::new(world);
    let mut event_writer = system_state.get_mut(world);

    for entity in events.read::<T>() {
        event_writer.send(InsertComponentEvent::<T>::new(entity));
    }
}

fn insert_asset_entry_event(world: &mut BevyWorld, events: &InsertComponentEvents<World>) {
    let mut system_state: SystemState<(
        ResMut<AssetMetadataStore>,
        ResMut<AssetRefProcessor>,
        Res<AssetCache>,
        Query<&AssetEntry>,
    )> = SystemState::new(world);
    let (mut metadata_store, mut asset_ref_processor, asset_cache, asset_entry_q) =
        system_state.get_mut(world);

    let mut aggregated_pending_asset_ref_insert_events = Vec::new();
    for entity in events.read::<AssetEntry>() {
        let Ok(asset_entry) = asset_entry_q.get(entity) else {
            panic!("Shouldn't happen");
        };
        let asset_id = *asset_entry.asset_id;
        info!(
            "received Asset Entry from World Server! (entity: {:?}, asset_id: {:?})",
            entity, asset_id
        );
        let mut pending_asset_ref_insert_events = asset_ref_processor.handle_add_asset_entry(
            &mut metadata_store,
            &asset_cache,
            &entity,
            &asset_id,
        );
        aggregated_pending_asset_ref_insert_events.append(&mut pending_asset_ref_insert_events);
    }

    if aggregated_pending_asset_ref_insert_events.is_empty() {
        return;
    }
    world.resource_scope(|world, asset_ref_processor: Mut<AssetRefProcessor>| {
        for (asset_processor_id, entity, typed_asset_id) in
            aggregated_pending_asset_ref_insert_events
        {
            let asset_processor = asset_ref_processor
                .get_asset_processor_ref(&asset_processor_id)
                .unwrap();
            asset_processor.deferred_process(world, &entity, &typed_asset_id);
        }
    });
}

fn insert_asset_ref_event<T: AssetProcessor>(
    world: &mut BevyWorld,
    events: &InsertComponentEvents<World>,
) {
    let mut list_of_events = Vec::new();

    let mut system_state: SystemState<(
        WorldClient,
        ResMut<AssetMetadataStore>,
        ResMut<AssetRefProcessor>,
        Res<AssetCache>,
        Query<&AssetEntry>,
        Query<&AssetRef<T>>,
    )> = SystemState::new(world);
    let (
        client,
        mut metadata_store,
        mut asset_ref_processor,
        asset_cache,
        asset_entry_q,
        asset_ref_q,
    ) = system_state.get_mut(world);

    for entity in events.read::<AssetRef<T>>() {
        let mut output = AssetRefProcessor::insert_asset_ref_events::<T>(
            &client,
            &asset_cache,
            &mut metadata_store,
            &mut asset_ref_processor,
            &asset_entry_q,
            &asset_ref_q,
            &entity,
        );
        list_of_events.append(&mut output);
    }

    //

    for (entity, typed_asset_id) in list_of_events {
        T::process(world, &entity, &typed_asset_id);
    }
}
