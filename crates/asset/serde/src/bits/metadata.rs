use naia_serde::{BitWriter, SerdeInternal as Serde};

use asset_id::{AssetType, ETag};

#[derive(Serde, Eq, PartialEq, Clone)]
pub struct AssetMetadataSerde {
    pub etag: ETag,
    pub asset_type: AssetType,
}

impl AssetMetadataSerde {
    pub fn new(etag: ETag, asset_type: AssetType) -> Self {
        Self { etag, asset_type }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = BitWriter::new();

        self.ser(&mut writer);

        writer.to_bytes().to_vec()
    }
}
