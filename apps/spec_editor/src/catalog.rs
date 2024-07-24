use asset_id::AssetId;

pub(crate) struct AssetCatalog;

impl AssetCatalog {
    pub(crate) fn avatar_model() -> AssetId {
        AssetId::from_str("dx8pye").unwrap()
    }

    pub(crate) fn avatar_idle_animation() -> AssetId {
        AssetId::from_str("m3y2n3").unwrap()
    }

    pub(crate) fn avatar_walk_animation() -> AssetId {
        AssetId::from_str("1x87se").unwrap()
    }
}