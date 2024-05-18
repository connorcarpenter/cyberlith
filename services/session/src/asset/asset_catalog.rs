use asset_id::AssetId;

pub struct AssetCatalog;

impl AssetCatalog {
    pub fn game_main_menu_ui() -> AssetId {
        AssetId::from_str("kmqkp9").unwrap()
    }
}