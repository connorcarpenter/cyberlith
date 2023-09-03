use input::Key;
use std::collections::HashMap;

pub struct KeyActionMap<A: Clone + Copy> {
    map: HashMap<Key, A>,
}

impl<A: Clone + Copy> KeyActionMap<A> {
    pub fn init(keys: Vec<(Key, A)>) -> Self {
        let mut state = HashMap::new();

        for (key, action) in keys {
            state.insert(key, action);
        }

        Self { map: state }
    }

    pub fn key_to_action(&self, key: Key) -> Option<A> {
        self.map.get(&key).copied()
    }
}
