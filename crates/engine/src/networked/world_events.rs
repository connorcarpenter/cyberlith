use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{
    change_detection::{Mut, ResMut},
    entity::Entity,
    event::{Event, EventReader},
    prelude::{Query, World as BevyWorld},
    system::{Res, SystemState},
};

use naia_bevy_client::events::{DespawnEntityEvent, InsertComponentEvents, SpawnEntityEvent};

use asset_id::{AssetId, AssetType};
use asset_loader::AssetMetadataStore;
use logging::info;

use world_server_naia_proto::components::{Alt1, AssetEntry, AssetRef, Main, Position};

use crate::{
    asset_cache::AssetCache,
    networked::{
        asset_ref_processor::{AssetProcessor, AssetRefProcessor},
        client_markers::World,
        component_events::{
            component_events_startup, get_component_events, AppRegisterComponentEvents,
            InsertComponentEvent, RemoveComponentEvent, UpdateComponentEvent,
        },
        connection_manager::ConnectionManager,
    },
    world::{WorldClient, WorldDespawnEntityEvent, WorldSpawnEntityEvent},
};

pub type WorldInsertComponentEvent<C> = InsertComponentEvent<World, C>;
pub type WorldUpdateComponentEvent<C> = UpdateComponentEvent<World, C>;
pub type WorldRemoveComponentEvent<C> = RemoveComponentEvent<World, C>;

pub struct WorldEventsPlugin;

impl Plugin for WorldEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ConnectionManager::handle_world_connect_events)
            .add_systems(Update, spawn_entity_events)
            .add_event::<WorldSpawnEntityEvent>()
            .add_systems(Update, despawn_entity_events)
            .add_event::<WorldDespawnEntityEvent>()
            .add_systems(Startup, component_events_startup::<World>)
            .add_systems(Update, component_events_update)
            // component events
            .add_component_events::<World, Position>()
            // asset events
            .add_event::<InsertAssetRefEvent<Main>>()
            .add_event::<InsertAssetRefEvent<Alt1>>();
    }
}

// used as a system
fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent<World>>) {
    for _event in event_reader.read() {
        // info!("spawned entity");
    }
}

// used as a system
fn despawn_entity_events(mut event_reader: EventReader<DespawnEntityEvent<World>>) {
    for _event in event_reader.read() {
        // info!("despawned entity");
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

// used as a system
pub fn component_events_update(world: &mut BevyWorld) {
    // insert & asset events

    for events in get_component_events::<World>(world) {
        info!("received world events: [");

        if events.is_insert() {
            // asset events
            insert_asset_entry_event(world, events.as_insert());
            insert_asset_ref_event::<Main>(world, events.as_insert());
            insert_asset_ref_event::<Alt1>(world, events.as_insert());
        }

        // component events
        events.process::<Position>(world);

        info!("]");
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
