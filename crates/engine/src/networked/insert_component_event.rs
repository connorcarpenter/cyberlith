use bevy_ecs::{
    change_detection::Mut,
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    prelude::{Resource, World as BevyWorld},
    system::SystemState,
};

use naia_bevy_client::{events::InsertComponentEvents, Replicate};

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

pub fn insert_component_events_startup<T: Send + Sync + 'static>(world: &mut BevyWorld) {
    let initial_state: SystemState<EventReader<InsertComponentEvents<T>>> = SystemState::new(world);
    // info!("insert_component_events_startup()");
    world.insert_resource(CachedInsertComponentEventsState {
        event_state: initial_state,
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
        // info!("in insert_component_event()");
        event_writer.send(InsertComponentEvent::<T, C>::new(entity));
    }
}
