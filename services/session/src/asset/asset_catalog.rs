use asset_id::AssetId;

pub struct AssetCatalog;

impl AssetCatalog {
    pub fn game_main_menu_ui() -> AssetId {
        AssetId::from_str("kmqkp9").unwrap()
    }

    pub fn game_host_match_ui() -> AssetId {
        AssetId::from_str("htytzu").unwrap()
    }

    pub fn game_global_chat_ui() -> AssetId {
        AssetId::from_str("ngffab").unwrap()
    }

    pub fn game_global_chat_list() -> AssetId {
        AssetId::from_str("ws8m4d").unwrap()
    }

    pub fn game_global_chat_list_item() -> AssetId {
        AssetId::from_str("ddbxab").unwrap()
    }
}
