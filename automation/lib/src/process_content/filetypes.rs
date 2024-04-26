use serde::{Deserialize, Serialize};

use asset_id::ETag;

use crate::process_content::error::FileIoError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcessedFileMeta {
    name: String,
    etag: String,
    hash: Vec<u8>,
}

impl ProcessedFileMeta {
    pub fn new(name: &str, etag: ETag, hash: Vec<u8>) -> Self {
        Self {
            name: name.to_string(),
            etag: etag.as_string(),
            hash,
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn hash(&self) -> &[u8] {
        &self.hash
    }

    pub fn etag(&self) -> ETag {
        ETag::from_str(&self.etag).unwrap()
    }

    pub fn write(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).unwrap()
    }

    pub fn read(bytes: &[u8]) -> Result<Self, FileIoError> {
        serde_json::from_slice(bytes).map_err(|e| FileIoError::Message(e.to_string()))
    }
}
