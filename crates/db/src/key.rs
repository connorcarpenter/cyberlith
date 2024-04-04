use std::hash::Hash;

use serde::{de::DeserializeOwned, Serialize};

pub trait DbTableKey: 'static {
    type Key: DbRowKey;
    type Value: DbRowValue;
}

pub trait DbRowKey: Send + Sync + Clone + Copy + Eq + PartialEq + Ord + PartialOrd + Hash + From<u64> + Into<u64> + Serialize + DeserializeOwned {}
pub trait DbRowValue: Send + Sync + Serialize + DeserializeOwned {}