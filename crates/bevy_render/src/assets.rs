use std::{marker::PhantomData, default::Default};

use bevy_ecs::{component::Component, prelude::Resource};

#[derive(Default, Clone, Component)]
pub struct Handle<T> {
    phantom_t: PhantomData<T>
}

impl<T> Handle<T> {
    pub fn new() -> Self {
        Self {
            phantom_t: PhantomData
        }
    }
}

#[derive(Resource)]
pub struct Assets<T> {
    phantom_t: PhantomData<T>
}

impl<T> Assets<T> {
    pub fn add(&mut self, t: T) -> Handle<T> {
        Handle::new()
    }
}