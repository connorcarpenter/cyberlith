use std::collections::HashMap;
use std::default::Default;

use bevy_ecs::prelude::Resource;

use super::Handle;

#[derive(Default, Resource)]
pub struct Assets<T> {
    assets: HashMap<u64, T>,
    last_id: u64,
}

impl<T> Assets<T> {
    pub fn add(&mut self, t: T) -> Handle<T> {
        let handle = Handle::new(self.last_id);
        self.assets.insert(self.last_id, t);
        self.last_id += 1;
        handle
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.assets.get(&handle.id)
    }
}
