use std::{collections::HashMap, default::Default};

use crate::AssetHandle;

#[derive(Default)]
pub struct AssetStorage<T> {
    data_map: HashMap<AssetHandle<T>, T>,
}

impl<T> AssetStorage<T> {
    pub fn insert(&mut self, asset_handle: AssetHandle<T>, data: T) {
        if self.data_map.contains_key(&asset_handle) {
            panic!("Asset with id {:?} already exists", asset_handle.asset_id());
        }
        self.data_map.insert(asset_handle, data);
    }

    pub fn has(&self, asset_handle: &AssetHandle<T>) -> bool {
        self.data_map.contains_key(asset_handle)
    }

    pub fn get(&self, asset_handle: &AssetHandle<T>) -> Option<&T> {
        self.data_map.get(asset_handle)
    }

    pub fn get_mut(&mut self, asset_handle: &AssetHandle<T>) -> Option<&mut T> {
        self.data_map.get_mut(asset_handle)
    }
}
