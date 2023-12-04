use std::{
    cmp::Ordering,
    default::Default,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use bevy_ecs::component::Component;

#[derive(Default, Component, Debug)]
pub struct Handle<T> {
    pub id: u64,
    phantom_t: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            phantom_t: PhantomData,
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            phantom_t: PhantomData,
        }
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.id, state);
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Handle<T> {}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}
