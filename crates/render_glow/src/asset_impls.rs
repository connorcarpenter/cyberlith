use std::collections::HashMap;
use std::default::Default;
use std::marker::PhantomData;

use bevy_ecs::prelude::Resource;

use render_api::Handle;

#[derive(Resource)]
pub struct AssetImpls<T, U> {
    assets: HashMap<u64, U>,
    phantom_t: PhantomData<T>,
}

impl<T, U> AssetImpls<T, U> {
    pub fn insert(&mut self, handle: Handle<T>, i12n: U) {
        self.assets.insert(handle.id, i12n);
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&U> {
        self.assets.get(&handle.id)
    }
}

impl<T, U> Default for AssetImpls<T, U> {
    fn default() -> Self {
        Self {
            assets: HashMap::new(),
            phantom_t: PhantomData,
        }
    }
}