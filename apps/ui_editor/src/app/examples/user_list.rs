use std::collections::BTreeMap;

use game_engine::{ui::{UiManager, UiHandle}, asset::AssetManager};

use crate::app::examples::{UserListState};

pub(crate) fn setup_user_list_test_case(
    ui_manager: &mut UiManager,
    asset_manager: &AssetManager,
    main_menu_ui_handle: &UiHandle,
    user_list_item_ui_handle: &UiHandle,
) -> UserListState {
    let mut user_list_state = UserListState::new(user_list_item_ui_handle);

    // setup user list
    user_list_state
        .user_list_ui_ext
        .set_container_ui(ui_manager, main_menu_ui_handle, "user_list");

    // setup users
    user_list_state.users = setup_users();

    user_list_state.sync_user_collections(ui_manager, asset_manager);

    user_list_state
}

fn setup_users() -> BTreeMap<u32, String> {
    let mut users = Vec::new();
    users.push("tom");
    users.push("ben");
    users.push("andrew");
    users.push("joe");
    users.push("jane");
    users.push("sarah");
    users.push("jim");
    users.push("bob");

    let mut user_map = BTreeMap::<u32, String>::new();

    for user in users {
        user_map.insert(user_map.len() as u32, user.to_string());
    }

    user_map
}