use naia_bevy_server::{Server, UserKey};

use asset_id::AssetId;
use bevy_http_client::HttpClient;

use crate::asset::asset_manager::AssetManager;

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

    pub fn game_global_chat_list_item() -> AssetId {
        AssetId::from_str("ddbxab").unwrap()
    }

    pub fn text_icon() -> AssetId {
        AssetId::from_str("34mvvk").unwrap()
    }

    pub fn password_eye_icon() -> AssetId {
        AssetId::from_str("qbgz5j").unwrap()
    }
}

pub(crate) fn user_load_default_assets(
    server: &mut Server,
    http_client: &mut HttpClient,
    asset_manager: &mut AssetManager,
    user_key: &UserKey,
) {
    for asset_id in [
        AssetCatalog::text_icon(),
        AssetCatalog::password_eye_icon(),
        AssetCatalog::game_main_menu_ui(),
        AssetCatalog::game_host_match_ui(),
        AssetCatalog::game_global_chat_ui(),
        AssetCatalog::game_global_chat_list_item()
    ]
        .iter()
    {
        asset_manager.load_user_asset(server, http_client, *user_key, asset_id);
    }
}