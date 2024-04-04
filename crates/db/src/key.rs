use std::hash::Hash;

use serde::{de::DeserializeOwned, Serialize};

pub trait DbTableKey: 'static {
    type Key: DbRowKey;
    type Value: DbRowValue<Key = Self::Key>;

    fn repo_name() -> &'static str;
}

pub trait DbRowKey: Send + Sync + Clone + Copy + Eq + PartialEq + Ord + PartialOrd + Hash + From<u64> + Into<u64> + Serialize + DeserializeOwned {}
pub trait DbRowValue: Send + Sync + Serialize + DeserializeOwned {
    type Key: DbRowKey;

    fn get_key(&self) -> <Self as DbRowValue>::Key;
}