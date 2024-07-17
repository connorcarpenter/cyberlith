use bevy_ecs::{
    event::{Event, Events},
    world::World,
};

pub enum UiGlobalEvent {
    Copied(String),
    PassThru,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNodeEvent {
    Clicked,
}

pub struct UiNodeEventHandler {
    inner: Box<dyn UiNodeEventHandlerTrait>,
}

impl UiNodeEventHandler {
    pub fn new<T: Event + Default>() -> Self {
        Self {
            inner: Box::new(UiNodeEventHandlerImpl::<T>::new()),
        }
    }

    pub fn custom(inner: impl UiNodeEventHandlerTrait) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    pub fn handle(&self, world: &mut World, event: UiNodeEvent) {
        self.inner.handle(world, event);
    }
}

impl Clone for UiNodeEventHandler {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone_box(),
        }
    }
}

pub trait UiNodeEventHandlerTrait: Send + Sync + 'static {
    fn handle(&self, world: &mut World, event: UiNodeEvent);
    fn clone_box(&self) -> Box<dyn UiNodeEventHandlerTrait>;
}

struct UiNodeEventHandlerImpl<T: Event + Default> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Event + Default> UiNodeEventHandlerImpl<T> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Event + Default> Clone for UiNodeEventHandlerImpl<T> {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<T: Event + Default> UiNodeEventHandlerTrait for UiNodeEventHandlerImpl<T> {
    fn handle(&self, world: &mut World, event: UiNodeEvent) {
        match event {
            UiNodeEvent::Clicked => {
                let mut event_writer = world.get_resource_mut::<Events<T>>().unwrap();
                event_writer.send(T::default());
            }
        }
    }

    fn clone_box(&self) -> Box<dyn UiNodeEventHandlerTrait> {
        Box::new(self.clone())
    }
}
