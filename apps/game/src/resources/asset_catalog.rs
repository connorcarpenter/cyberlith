use game_engine::asset::AssetId;

pub struct AssetCatalog;

impl AssetCatalog {
    pub fn game_main_menu_ui() -> AssetId {
        AssetId::from_str("kmqkp9").unwrap()
    }

    pub fn game_host_match_ui() -> AssetId {
        AssetId::from_str("htytzu").unwrap()
    }
}