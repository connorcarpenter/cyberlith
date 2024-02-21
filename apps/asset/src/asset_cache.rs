use std::collections::{HashMap, VecDeque};

use log::{info, warn};

/// Stores asset data in RAM, but falls back to disk if over capacity
pub struct AssetCache {
    capacity_kb: u32,
    current_size_kb: u32,
    data_map: HashMap<String, Vec<u8>>,
    access_deque: VecDeque<String>,
}

impl AssetCache {
    pub fn new(capacity_kb: u32) -> Self {
        Self {
            capacity_kb,
            current_size_kb: 0,
            data_map: HashMap::new(),
            access_deque: VecDeque::new(),
        }
    }

    pub fn load(&mut self, path: &str) -> Option<Vec<u8>> {
        if let Some(data) = self.data_map.get(path) {
            // data is already in the cache, return it!

            info!("Cache hit: `{}`", path);

            // update lru order
            self.access_deque.retain(|x| x != path); // costly, O(n)
            self.access_deque.push_back(path.to_string());

            return Some(data.clone());
        }

        let Ok(bytes) = std::fs::read(&path) else {
            warn!("Failed to load file: `{}`", &path);
            return None;
        };
        let byte_count = bytes.len();
        let kb_count = bytes_to_kb(byte_count);

        self.data_map.insert(path.to_string(), bytes.clone());
        self.current_size_kb += kb_count;
        self.access_deque.push_back(path.to_string());

        info!("Cache miss. Loading `{}` (size: {:?} kb) ... Current cache size: {:?} kb, max: {:?} kb", path, kb_count, self.current_size_kb, self.capacity_kb);

        if self.current_size_kb > self.capacity_kb {
            info!("Cache over capacity, unloading until under capacity... current: {:?} kb, max: {:?} kb", self.current_size_kb, self.capacity_kb);
            self.unload_until_under_capacity();
        }

        return Some(bytes);
    }

    fn unload_until_under_capacity(&mut self) {
        loop {
            self.unload_lru_file();
            if self.current_size_kb <= self.capacity_kb {
                return;
            }
        }
    }

    fn unload_lru_file(&mut self) {
        let Some(oldest_path) = self.access_deque.pop_front() else {
            panic!("No items in the cache to unload, even though over capacity!");
        };

        let data = self.data_map.remove(&oldest_path).unwrap();
        let byte_count = data.len();
        let kb_count = bytes_to_kb(byte_count);

        self.current_size_kb -= kb_count;

        info!(
            "Unloaded LRU file: {:?} (size: {:?})... current: {:?} kb, max: {:?} kb",
            oldest_path, kb_count, self.current_size_kb, self.capacity_kb
        );
    }
}

fn bytes_to_kb(bytes: usize) -> u32 {
    bytes as u32
    // (bytes / 1024) as u32 // TODO: uncomment this, it's just for simulation!
}
