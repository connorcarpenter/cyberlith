use bevy_ecs::{
    change_detection::Mut,
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    prelude::{Resource, World as BevyWorld},
    system::SystemState,
};

use naia_bevy_client::{events::{RemoveComponentEvents, UpdateComponentEvents, InsertComponentEvents}, Replicate, Tick};

// Insert Component Events

#[derive(Event)]
pub struct InsertComponentEvent<T, C: Replicate> {
    pub entity: Entity,
    phantom_t: std::marker::PhantomData<T>,
    phantom_c: std::marker::PhantomData<C>,
}

impl<T, C: Replicate> InsertComponentEvent<T, C> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            phantom_t: std::marker::PhantomData,
            phantom_c: std::marker::PhantomData,
        }
    }
}

#[derive(Resource)]
pub struct CachedInsertComponentEventsState<T: Send + Sync + 'static> {
    event_state: SystemState<EventReader<'static, 'static, InsertComponentEvents<T>>>,
}

pub fn component_events_startup<T: Send + Sync + 'static> (
    world: &mut BevyWorld,
) {
    let insert_event_state: SystemState<EventReader<InsertComponentEvents<T>>> = SystemState::new(world);
    world.insert_resource(CachedInsertComponentEventsState {
        event_state: insert_event_state,
    });

    let update_event_state: SystemState<EventReader<UpdateComponentEvents<T>>> = SystemState::new(world);
    world.insert_resource(CachedUpdateComponentEventsState {
        event_state: update_event_state,
    });

    let remove_event_state: SystemState<EventReader<RemoveComponentEvents<T>>> = SystemState::new(world);
    world.insert_resource(CachedRemoveComponentEventsState {
        event_state: remove_event_state,
    });
}

// this is not a system! It should be wrapped!
pub fn insert_component_events<T: Clone + Send + Sync + 'static>(
    world: &mut BevyWorld,
) -> Vec<InsertComponentEvents<T>> {
    let mut events_collection: Vec<InsertComponentEvents<T>> = Vec::new();

    world.resource_scope(
        |world, mut events_reader_state: Mut<CachedInsertComponentEventsState<T>>| {
            let mut events_reader = events_reader_state.event_state.get_mut(world);

            for events in events_reader.read() {
                let events_clone: InsertComponentEvents<T> = Clone::clone(events);
                // info!("insert_component_events() events");
                events_collection.push(events_clone);
            }
        },
    );

    events_collection
}

pub fn insert_component_event<T: Send + Sync + 'static, C: Replicate>(
    world: &mut BevyWorld,
    events: &InsertComponentEvents<T>,
) {
    let mut system_state: SystemState<EventWriter<InsertComponentEvent<T, C>>> =
        SystemState::new(world);
    let mut event_writer = system_state.get_mut(world);

    for entity in events.read::<C>() {
        event_writer.send(InsertComponentEvent::<T, C>::new(entity));
    }
}

// Update Component Events

#[derive(Event)]
pub struct UpdateComponentEvent<T, C: Replicate> {
    pub tick: Tick,
    pub entity: Entity,
    phantom_t: std::marker::PhantomData<T>,
    phantom_c: std::marker::PhantomData<C>,
}

impl<T, C: Replicate> UpdateComponentEvent<T, C> {
    pub fn new(tick: Tick, entity: Entity) -> Self {
        Self {
            tick,
            entity,
            phantom_t: std::marker::PhantomData,
            phantom_c: std::marker::PhantomData,
        }
    }
}

#[derive(Resource)]
pub struct CachedUpdateComponentEventsState<T: Send + Sync + 'static> {
    event_state: SystemState<EventReader<'static, 'static, UpdateComponentEvents<T>>>,
}

// this is not a system! It should be wrapped!
pub fn update_component_events<T: Clone + Send + Sync + 'static>(
    world: &mut BevyWorld,
) -> Vec<UpdateComponentEvents<T>> {
    let mut events_collection: Vec<UpdateComponentEvents<T>> = Vec::new();

    world.resource_scope(
        |world, mut events_reader_state: Mut<CachedUpdateComponentEventsState<T>>| {
            let mut events_reader = events_reader_state.event_state.get_mut(world);

            for events in events_reader.read() {
                let events_clone: UpdateComponentEvents<T> = Clone::clone(events);

                events_collection.push(events_clone);
            }
        },
    );

    events_collection
}

pub fn update_component_event<T: Send + Sync + 'static, C: Replicate>(
    world: &mut BevyWorld,
    events: &UpdateComponentEvents<T>,
) {
    let mut system_state: SystemState<EventWriter<UpdateComponentEvent<T, C>>> =
        SystemState::new(world);
    let mut event_writer = system_state.get_mut(world);

    for (tick, entity) in events.read::<C>() {
        event_writer.send(UpdateComponentEvent::<T, C>::new(tick, entity));
    }
}

// Remove Component Events

#[derive(Event)]
pub struct RemoveComponentEvent<T, C: Replicate> {
    pub entity: Entity,
    phantom_t: std::marker::PhantomData<T>,
    pub component: C,
}

impl<T, C: Replicate> RemoveComponentEvent<T, C> {
    pub fn new(entity: Entity, component: C) -> Self {
        Self {
            entity,
            phantom_t: std::marker::PhantomData,
            component,
        }
    }
}

#[derive(Resource)]
pub struct CachedRemoveComponentEventsState<T: Send + Sync + 'static> {
    event_state: SystemState<EventReader<'static, 'static, RemoveComponentEvents<T>>>,
}

// this is not a system! It should be wrapped!
pub fn remove_component_events<T: Clone + Send + Sync + 'static>(
    world: &mut BevyWorld,
) -> Vec<RemoveComponentEvents<T>> {
    let mut events_collection: Vec<RemoveComponentEvents<T>> = Vec::new();

    world.resource_scope(
        |world, mut events_reader_state: Mut<CachedRemoveComponentEventsState<T>>| {
            let mut events_reader = events_reader_state.event_state.get_mut(world);

            for events in events_reader.read() {
                let events_clone: RemoveComponentEvents<T> = Clone::clone(events);
                events_collection.push(events_clone);
            }
        },
    );

    events_collection
}

pub fn remove_component_event<T: Send + Sync + 'static, C: Replicate>(
    world: &mut BevyWorld,
    events: &RemoveComponentEvents<T>,
) {
    let mut system_state: SystemState<EventWriter<RemoveComponentEvent<T, C>>> =
        SystemState::new(world);
    let mut event_writer = system_state.get_mut(world);

    for (entity, component) in events.read::<C>() {
        event_writer.send(RemoveComponentEvent::<T, C>::new(entity, component));
    }
}