use std::collections::HashMap;
use crypto::U32Token;

use editor_proto::resources::FileKey;

pub type AssetId = U32Token;

pub struct AssetIdData {
    file_key: FileKey,
}

pub struct AssetIdStore {
    id_map: HashMap<AssetId, AssetIdData>,
    path_map: HashMap<String, AssetId>,
}

impl AssetIdStore {
    pub fn new() -> Self {
        Self {
            id_map: HashMap::new(),
            path_map: HashMap::new(),
        }
    }

    pub(crate) fn id_from_path(&self, file_path: &str) -> Option<AssetId> {
        self.path_map.get(file_path).copied()
    }

    pub(crate) fn generate_new_unique_id(&mut self) -> AssetId {
        let mut id = U32Token::get_random();
        while self.id_map.contains_key(&id) {
            id = U32Token::get_random();
        }
        id
    }

    pub(crate) fn get_path_and_name(&self, id: &AssetId) -> Option<(String, String)> {
        self.id_map.get(id).map(|data| (data.file_key.path().to_string(), data.file_key.name().to_string()))
    }
}