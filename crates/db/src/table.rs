use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use git::{repo_init, Repository};
use logging::info;

use crate::{
    error::DbError,
    git_ops::{create_new_file,update_file, pull_repo_get_all_files, update_nextid},
    DbRowValue, DbTableKey,
};

// Table trait
pub trait Table: Send + Sync {
    fn to_any_ref(&self) -> &dyn Any;
    fn to_any_mut(&mut self) -> &mut dyn Any;
}

// TableImpl
pub struct TableImpl<K: DbTableKey> {
    root_path: String,
    repo: Arc<Mutex<Repository>>,

    next_id: u64,
    next_key_has_changed: bool,
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
        let (root_path, git_repo) = repo_init(K::repo_name());
        let files = pull_repo_get_all_files(&root_path, &git_repo);

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
                // ".json" => {
                _ => {
                    let Ok(file_value) = serde_json::from_slice::<K::Value>(&file.bytes) else {
                        panic!("failed to deserialize file: {}", file.name);
                    };
                    let file_key = file_value.get_key();
                    store.insert(file_key, file_value);
                }
            }
        }

        Self {
            root_path,
            repo: Arc::new(Mutex::new(git_repo)),
            next_id,
            next_key_has_changed: false,
            store,
        }
    }

    pub fn insert(&mut self, mut value: K::Value) -> Result<K::Key, DbError> {
        // get next key
        let key = self.get_next_key();
        value.set_key(key);

        // insert into in-memory store
        if self.store.contains_key(&key) {
            return Err(DbError::KeyAlreadyExists);
        }
        self.store.insert(key, value.clone());

        // upload to database
        {
            let repo = self.repo.lock().unwrap();
            create_new_file::<K>(&self.root_path, &repo, value);
        }

        // update nextkey
        self.update_nextid();

        Ok(key)
    }

    pub fn list(&self) -> Vec<(&K::Key, &K::Value)> {
        self.store.iter().collect()
    }

    pub fn get(&self, key: &K::Key) -> Option<&K::Value> {
        self.store.get(key)
    }

    pub fn get_mut<F: FnMut(&mut K::Value)>(&mut self, key: &K::Key, mut func: F) {
        {
            // change the file via closure
            let item_mut = self.store.get_mut(key).unwrap();
            func(item_mut);
        }

        // upload to database
        {
            let item_ref = self.store.get(key).unwrap();
            let repo = self.repo.lock().unwrap();
            update_file::<K>(&self.root_path, &repo, item_ref);
        }
    }

    pub fn remove(&mut self, _key: &K::Key) -> Option<K::Value> {
        // TODO: queue sync with actual datastore, this just modifies in-memory

        //self.store.remove(key)
        todo!()
    }

    fn get_next_key(&mut self) -> K::Key {
        let next_key = K::Key::from(self.next_id);
        self.next_id += 1;
        self.next_key_has_changed = true;
        next_key
    }

    fn update_nextid(&mut self) {
        if !self.next_key_has_changed {
            return;
        }
        self.next_key_has_changed = false;

        let repo = self.repo.lock().unwrap();
        update_nextid(&self.root_path, &repo, self.next_id);
    }
}
