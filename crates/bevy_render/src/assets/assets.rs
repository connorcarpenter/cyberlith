use std::{marker::PhantomData, default::Default};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use bevy_ecs::{component::Component, prelude::Resource};

use crate::Handle;

#[derive(Default, Resource)]
pub struct Assets<T> {
    assets: HashMap<u64, T>,
    last_id: u64,
}

impl<T> Assets<T> {

    pub fn add(&mut self, t: T) -> Handle<T> {
        let handle = Handle::new(self.last_id);
        self.last_id += 1;
        handle
    }
}