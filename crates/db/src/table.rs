use std::{collections::HashMap, any::Any};

use crate::DbTableKey;

// Table trait
pub trait Table: Send + Sync {
    fn to_any_ref(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
}

// TableImpl
pub struct TableImpl<K: DbTableKey> {
    next_id: u64,
    store: HashMap<K::Key, K::Value>,
}

impl<K: DbTableKey> Table for TableImpl<K> {
    fn to_any_ref(&self) -> &dyn Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<K: DbTableKey> TableImpl<K> {
    pub fn new(repo_name: &str) -> Self {
        // lot to do here ..
        Self {
            next_id: 0,
            store: HashMap::new(),
        }
    }

    fn increment_key(&mut self) {
        self.next_id += 1;
    }

    pub fn insert(&mut self, value: K::Value) -> K::Key {
        // TODO: queue sync with actual datastore, this just modifies in-memory

        let key = K::Key::from(self.next_id);
        self.increment_key();
        self.store.insert(key, value);
        key
    }

    pub fn get(&self, key: &K::Key) -> Option<&K::Value> {
        self.store.get(key)
    }

    pub fn get_mut(&mut self, key: &K::Key) -> Option<&mut K::Value> {
        // TODO: queue sync with actual datastore, this just modifies in-memory

        self.store.get_mut(key)
    }

    pub fn remove(&mut self, key: &K::Key) -> Option<K::Value> {
        // TODO: queue sync with actual datastore, this just modifies in-memory

        self.store.remove(key)
    }
}

