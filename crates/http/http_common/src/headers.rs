use std::collections::hash_map::Iter;
use std::collections::HashMap;

#[derive(Clone)]
pub struct HeaderStore {
    map: HashMap<String, Vec<String>>,
}

impl HeaderStore {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: String) {
        let key = key.to_ascii_lowercase();

        self.map.entry(key).or_insert_with(Vec::new).push(value);
    }

    pub fn remove_all(&mut self, key: &str) {
        let key = key.to_ascii_lowercase();
        self.map.remove(&key);
    }

    pub fn has(&self, key: &str) -> bool {
        let key = key.to_ascii_lowercase();
        self.map.contains_key(&key)
    }

    pub fn get(&self, key: &str) -> Option<&Vec<String>> {
        let key = key.to_ascii_lowercase();
        self.map.get(&key)
    }

    pub fn iter(&self) -> Iter<'_, String, Vec<String>> {
        self.map.iter()
    }
}