use std::hash::{Hash, Hasher};

use asset_id::AssetId;

#[derive(Debug)]
pub struct UiHandle {
    asset_id: AssetId,
}

impl UiHandle {
    pub fn new(asset_id: AssetId) -> Self {
        Self { asset_id }
    }

    pub fn asset_id(&self) -> AssetId {
        self.asset_id
    }
}

impl Hash for UiHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.asset_id.hash(state);
    }
}

impl PartialEq<Self> for UiHandle {
    fn eq(&self, other: &Self) -> bool {
        self.asset_id == other.asset_id
    }
}

impl Eq for UiHandle {}

impl Clone for UiHandle {
    fn clone(&self) -> Self {
        Self {
            asset_id: self.asset_id,
        }
    }
}

impl Copy for UiHandle {}
