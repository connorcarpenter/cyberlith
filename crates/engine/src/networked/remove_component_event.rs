use bevy_ecs::{
    change_detection::Mut,
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    prelude::{Resource, World as BevyWorld},
    system::SystemState,
};

use naia_bevy_client::{events::RemoveComponentEvents, Replicate};

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

pub fn remove_component_events_startup<T: Send + Sync + 'static>(world: &mut BevyWorld) {
    let initial_state: SystemState<EventReader<RemoveComponentEvents<T>>> = SystemState::new(world);
    // info!("remove_component_events_startup()");
    world.insert_resource(CachedRemoveComponentEventsState {
        event_state: initial_state,
    });
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
                // info!("remove_component_events() events");
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
        // info!("in remove_component_event()");
        event_writer.send(RemoveComponentEvent::<T, C>::new(entity, component));
    }
}
