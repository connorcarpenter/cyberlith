use std::{collections::HashMap, default::Default, marker::PhantomData};

use bevy_ecs::system::Resource;

use crate::Handle;

#[derive(Resource)]
pub struct SideStorage<T, U> {
    data_map: HashMap<Handle<T>, U>,
    phantom_t: PhantomData<T>,
}

impl<T, U> SideStorage<T, U> {
    pub fn insert(&mut self, handle: Handle<T>, i12n: U) {
        self.data_map.insert(handle, i12n);
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&U> {
        self.data_map.get(&handle)
    }

    pub fn remove(&mut self, handle: &Handle<T>) -> Option<U> {
        self.data_map.remove(handle)
    }
}

impl<T, U> Default for SideStorage<T, U> {
    fn default() -> Self {
        Self {
            data_map: HashMap::new(),
            phantom_t: PhantomData,
        }
    }
}
