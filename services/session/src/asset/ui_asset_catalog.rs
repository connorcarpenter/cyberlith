use naia_bevy_server::{Server, UserKey};

use asset_id::AssetId;
use bevy_http_client::HttpClient;

use crate::asset::asset_manager::AssetManager;

pub struct UiAssetCatalog;

impl UiAssetCatalog {
    pub fn game_main_menu_ui() -> AssetId {
        AssetId::from_str("kmqkp9").unwrap()
    }

    pub fn game_host_match_ui() -> AssetId {
        AssetId::from_str("htytzu").unwrap()
    }

    pub fn game_join_match_ui() -> AssetId {
        AssetId::from_str("qqxe6s").unwrap()
    }

    pub fn game_join_match_lobby_list_item_ui() -> AssetId {
        AssetId::from_str("pup52m").unwrap()
    }

    pub fn game_global_chat_ui() -> AssetId {
        AssetId::from_str("ngffab").unwrap()
    }

    pub fn game_global_chat_day_divider_item_ui() -> AssetId {
        AssetId::from_str("3wnz6n").unwrap()
    }

    pub fn game_global_chat_username_and_message_item_ui() -> AssetId {
        AssetId::from_str("ddbxab").unwrap()
    }

    pub fn game_global_chat_message_item_ui() -> AssetId {
        AssetId::from_str("cxc6zk").unwrap()
    }

    pub fn game_user_list_item_ui() -> AssetId {
        AssetId::from_str("8ywqfp").unwrap()
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
        UiAssetCatalog::text_icon(),
        UiAssetCatalog::password_eye_icon(),
        UiAssetCatalog::game_main_menu_ui(),
        UiAssetCatalog::game_host_match_ui(),
        UiAssetCatalog::game_join_match_ui(),
        UiAssetCatalog::game_join_match_lobby_list_item_ui(),
        UiAssetCatalog::game_global_chat_ui(),
        UiAssetCatalog::game_global_chat_day_divider_item_ui(),
        UiAssetCatalog::game_global_chat_username_and_message_item_ui(),
        UiAssetCatalog::game_global_chat_message_item_ui(),
        UiAssetCatalog::game_user_list_item_ui(),
    ]
    .iter()
    {
        asset_manager.load_user_asset(server, http_client, *user_key, asset_id);
    }
}
