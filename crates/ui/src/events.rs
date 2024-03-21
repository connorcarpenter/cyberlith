use bevy_ecs::{
    event::{Event, Events},
    world::World,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiEvent {
    Clicked,
}

pub struct UiEventHandler {
    inner: Box<dyn UiEventHandlerTrait>,
}

impl UiEventHandler {
    pub fn new<T: Event + Default>() -> Self {
        Self {
            inner: Box::new(UiEventHandlerImpl::<T>::new()),
        }
    }

    pub fn handle(&self, world: &mut World, event: UiEvent) {
        self.inner.handle(world, event);
    }
}

trait UiEventHandlerTrait: Send + Sync + 'static {
    fn handle(&self, world: &mut World, event: UiEvent);
}

struct UiEventHandlerImpl<T: Event + Default> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Event + Default> UiEventHandlerImpl<T> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Event + Default> UiEventHandlerTrait for UiEventHandlerImpl<T> {
    fn handle(&self, world: &mut World, event: UiEvent) {
        match event {
            UiEvent::Clicked => {
                let mut event_writer = world.get_resource_mut::<Events<T>>().unwrap();
                event_writer.send(T::default());
            }
        }
    }
}
