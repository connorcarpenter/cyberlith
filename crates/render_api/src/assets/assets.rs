use std::{
    any::{Any, TypeId},
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    default::Default,
    hash::{Hash, Hasher},
};

use bevy_ecs::system::Resource;

use super::Handle;

#[derive(Default, Resource)]
pub struct Assets<T> {
    keys: HashMap<(TypeId, u64), Handle<T>>,
    assets: HashMap<u64, T>,
    last_id: u64,
    added_ids: HashSet<u64>,
    changed_ids: HashSet<u64>,
    removed_ids: HashSet<u64>,
}

pub trait AssetHash<T>: Any + Hash + Into<T> {
    fn get_key(&self) -> (TypeId, u64) {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        (self.type_id(), hasher.finish())
    }
}

impl<T> Assets<T> {
    pub fn add<L: AssetHash<T>>(&mut self, asset: L) -> Handle<T> {
        let key = asset.get_key();

        if let Some(old_handle) = self.keys.get(&key) {
            //info!("getting old handle");
            return old_handle.clone();
        }

        //info!("making new handle");
        let new_handle = self.add_inner(asset.into());
        self.keys.insert(key, new_handle.clone());
        new_handle
    }

    fn add_inner(&mut self, asset: T) -> Handle<T> {
        let handle = Handle::new(self.last_id);
        self.assets.insert(self.last_id, asset);
        self.added_ids.insert(self.last_id);
        self.last_id += 1;
        handle
    }

    pub fn remove(&mut self, handle: &Handle<T>) -> T {
        if !self.assets.contains_key(&handle.id) {
            panic!("Asset with id {} does not exist", handle.id);
        }
        self.removed_ids.insert(handle.id);
        self.assets.remove(&handle.id).unwrap()
    }

    pub fn set(&mut self, handle: &Handle<T>, t: T) {
        if !self.assets.contains_key(&handle.id) {
            panic!("Asset with id {} does not exist", handle.id);
        }
        self.assets.insert(handle.id, t);
        self.changed_ids.insert(handle.id);
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        self.assets.get(&handle.id)
    }

    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.changed_ids.insert(handle.id);
        self.assets.get_mut(&handle.id)
    }

    pub fn flush_added(&mut self) -> Vec<Handle<T>> {
        let output = (&self.added_ids)
            .into_iter()
            .map(|id| Handle::new(*id))
            .collect();
        self.added_ids.clear();
        output
    }

    pub fn flush_changed(&mut self) -> Vec<Handle<T>> {
        let output = (&self.changed_ids)
            .into_iter()
            .map(|id| Handle::new(*id))
            .collect();
        self.changed_ids.clear();
        output
    }

    pub fn flush_removed(&mut self) -> Vec<Handle<T>> {
        let output = (&self.removed_ids)
            .into_iter()
            .map(|id| Handle::new(*id))
            .collect();
        self.removed_ids.clear();
        output
    }
}
