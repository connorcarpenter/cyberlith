use std::hash::Hash;

use serde::{de::DeserializeOwned, Serialize};

pub trait DbTableKey: 'static {
    type Key: DbRowKey;
    type Value: DbRowValue<Key = Self::Key>;

    fn repo_name() -> &'static str;
}

pub trait DbRowKey: Send + Sync + Clone + Copy + Eq + PartialEq + Ord + PartialOrd + Hash + From<u64> + Into<u64> + Serialize + DeserializeOwned {}
pub trait DbRowValue: Send + Sync + Clone + Serialize + DeserializeOwned {
    type Key: DbRowKey;

    fn get_key(&self) -> <Self as DbRowValue>::Key;
    fn set_key(&mut self, key: <Self as DbRowValue>::Key);

    fn get_file_name(&self) -> String;
    fn get_insert_commit_message(&self) -> String;
    fn get_update_commit_message(&self) -> String;
    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self)
            .unwrap()
            .to_vec()
    }
}