use std::collections::hash_map::Iter;
use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct HeaderStore {
    map: HashMap<String, Vec<String>>,
}

impl HeaderStore {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, key: String, value: String) {
        let key = key.to_ascii_lowercase();

        self.map.entry(key).or_insert_with(Vec::new).push(value);
    }

    pub(crate) fn remove_all(&mut self, key: &str) {
        let key = key.to_ascii_lowercase();
        self.map.remove(&key);
    }

    pub(crate) fn has(&self, key: &str) -> bool {
        let key = key.to_ascii_lowercase();
        self.map.contains_key(&key)
    }

    pub(crate) fn get(&self, key: &str) -> Option<&Vec<String>> {
        let key = key.to_ascii_lowercase();
        self.map.get(&key)
    }

    pub(crate) fn iter(&self) -> Iter<'_, String, Vec<String>> {
        self.map.iter()
    }
}