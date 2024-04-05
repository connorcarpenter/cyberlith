use std::{collections::HashMap, any::Any};
use log::info;

use crate::{DbRowValue, DbTableKey};
use crate::git_ops::{create_new_file, pull_repo_get_all_files};

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
    pub fn init() -> Self {
        // lot to do here ..
        let files = pull_repo_get_all_files::<K>();

        let mut next_id: u64 = 0;
        let mut store = HashMap::new();

        for file in files {
            match file.name.as_str() {
                ".nextid" => {
                    let Ok(val) = serde_json::from_slice::<u64>(&file.bytes) else {
                        panic!("failed to deserialize file: {}", file.name);
                    };
                    info!("found next_id: {}", val);
                    next_id = val;
                }
                ".json" => {
                    let Ok(file_value) = serde_json::from_slice::<K::Value>(&file.bytes) else {
                        panic!("failed to deserialize file: {}", file.name);
                    };
                    let file_key = file_value.get_key();
                    store.insert(file_key, file_value);
                }
                _ => {
                    panic!("unknown file extension for: {}", file.name);
                }
            }
        }

        Self {
            next_id,
            store,
        }
    }

    fn get_next_key(&mut self) -> K::Key {
        let next_key = K::Key::from(self.next_id);
        self.next_id += 1;
        next_key
    }

    pub fn insert(&mut self, mut value: K::Value) -> K::Key {

        // get next key
        let key = self.get_next_key();
        value.set_key(key);

        // insert into in-memory store
        self.store.insert(key, value.clone());

        // upload to database
        //pub fn create_new_file(repo_name: &str, file_name: &str, file_contents: Vec<u8>, commit_message: &str);
        create_new_file::<K>(value);

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

