use std::{collections::HashMap, default::Default, marker::PhantomData};

use bevy_ecs::system::Resource;

use crate::Handle;

#[derive(Resource)]
pub struct AssetMapping<T, U> {
    assets: HashMap<Handle<T>, U>,
    phantom_t: PhantomData<T>,
}

impl<T, U> AssetMapping<T, U> {
    pub fn insert(&mut self, handle: Handle<T>, i12n: U) {
        self.assets.insert(handle, i12n);
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&U> {
        self.assets.get(&handle)
    }

    pub fn remove(&mut self, handle: &Handle<T>) -> Option<U> {
        self.assets.remove(handle)
    }
}

impl<T, U> Default for AssetMapping<T, U> {
    fn default() -> Self {
        Self {
            assets: HashMap::new(),
            phantom_t: PhantomData,
        }
    }
}
