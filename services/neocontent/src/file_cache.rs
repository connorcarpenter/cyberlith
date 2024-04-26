use std::collections::{HashMap, VecDeque};

use logging::{info, warn};

/// Stores file data in RAM, but falls back to disk if over capacity
pub struct FileCache {
    capacity_size_bytes: u32,
    current_size_bytes: u32,
    data_map: HashMap<String, Vec<u8>>,
    access_deque: VecDeque<String>,
}

impl FileCache {
    pub fn new(capacity_kb: u32) -> Self {
        Self {
            capacity_size_bytes: kb_to_bytes(capacity_kb),
            current_size_bytes: 0,
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
        let byte_count = bytes.len() as u32;
        // info!("{} : {} byte count", path, byte_count);

        self.data_map.insert(path.to_string(), bytes.clone());
        self.current_size_bytes += byte_count;
        self.access_deque.push_back(path.to_string());

        let file_kb_count = bytes_to_kb(byte_count);
        let current_kb_count = bytes_to_kb(self.current_size_bytes);
        let capacity_kb_count = bytes_to_kb(self.capacity_size_bytes);
        info!("Cache miss. Loading `{}` (size: {:?} kb) ... Current cache size: {:?} kb, max: {:?} kb", path, file_kb_count, current_kb_count, capacity_kb_count);

        if self.current_size_bytes > self.capacity_size_bytes {
            info!("Cache over capacity, unloading until under capacity... current: {:?} kb, max: {:?} kb", current_kb_count, capacity_kb_count);
            self.unload_until_under_capacity();
        }

        return Some(bytes);
    }

    fn unload_until_under_capacity(&mut self) {
        loop {
            self.unload_lru_file();
            if self.current_size_bytes <= self.capacity_size_bytes {
                return;
            }
        }
    }

    fn unload_lru_file(&mut self) {
        let Some(oldest_path) = self.access_deque.pop_front() else {
            panic!("No items in the cache to unload, even though over capacity!");
        };

        let data = self.data_map.remove(&oldest_path).unwrap();
        let byte_count = data.len() as u32;

        self.current_size_bytes -= byte_count;

        info!(
            "Unloaded LRU file: {:?} (size: {:?})... current: {:?} kb, max: {:?} kb",
            oldest_path,
            bytes_to_kb(byte_count),
            bytes_to_kb(self.current_size_bytes),
            bytes_to_kb(self.capacity_size_bytes)
        );
    }
}

fn bytes_to_kb(bytes: u32) -> u32 {
    (bytes / 1024)
}

fn kb_to_bytes(kb: u32) -> u32 {
    kb * 1024
}
