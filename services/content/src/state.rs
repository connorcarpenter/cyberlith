use crate::file_metadata_store::FileMetadata;
use crate::{file_cache::FileCache, file_metadata_store::FileMetadataStore};

pub struct State {
    file_metadata_store: FileMetadataStore,
    file_cache: FileCache,
}

impl State {
    pub fn new(file_cache_size_kb: u32, file_metadata_store: FileMetadataStore) -> Self {
        Self {
            file_metadata_store,
            file_cache: FileCache::new(file_cache_size_kb),
        }
    }

    pub fn get_metadata(&self, file_name: &str) -> Option<&FileMetadata> {
        self.file_metadata_store.get(file_name)
    }

    pub fn cache_load(&mut self, file_path: &str) -> Option<Vec<u8>> {
        self.file_cache.load(file_path)
    }
}
