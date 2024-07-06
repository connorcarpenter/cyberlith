use bevy_ecs::{
    change_detection::Mut,
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    prelude::{Resource, World as BevyWorld},
    system::SystemState,
};

use naia_bevy_client::{events::UpdateComponentEvents, Replicate, Tick};

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

pub fn update_component_events_startup<T: Send + Sync + 'static>(world: &mut BevyWorld) {
    let initial_state: SystemState<EventReader<UpdateComponentEvents<T>>> = SystemState::new(world);

    world.insert_resource(CachedUpdateComponentEventsState {
        event_state: initial_state,
    });
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
