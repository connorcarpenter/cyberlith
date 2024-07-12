use std::collections::BTreeMap;

use bevy_ecs::prelude::Resource;

use game_engine::{ui::{UiManager, UiHandle, extensions::{ListUiExt, ListUiExtItem}}, asset::AssetManager};

#[derive(Resource)]
pub struct UserListState {
    pub user_list_ui_ext: ListUiExt<u32>,
    pub users: BTreeMap<u32, String>,
    user_ui_handle: UiHandle,
}

impl UserListState {
    pub fn new(user_ui_handle: &UiHandle) -> Self {
        Self {
            user_list_ui_ext: ListUiExt::new(true),
            users: BTreeMap::new(),
            user_ui_handle: user_ui_handle.clone(),
        }
    }

    pub fn user_list_scroll_up(&mut self, ui_manager: &mut UiManager, asset_manager: &AssetManager) {
        self.user_list_ui_ext.scroll_up();

        self.sync_user_collections(ui_manager, asset_manager);
    }

    pub fn user_list_scroll_down(&mut self, ui_manager: &mut UiManager, asset_manager: &AssetManager) {
        self.user_list_ui_ext.scroll_down();

        self.sync_user_collections(ui_manager, asset_manager);
    }

    pub fn sync_user_collections(
        &mut self,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
    ) {
        // setup collection
        self.user_list_ui_ext.sync_with_collection(
            ui_manager,
            asset_manager,
            self.users.iter(),
            self.users.len(),
            |item_ctx, user_id, _| {

                let username = self.users.get(&user_id).unwrap();
                add_user_item(item_ctx, &self.user_ui_handle, username);
            },
        );
    }
}

fn add_user_item(item_ctx: &mut ListUiExtItem<u32>, ui: &UiHandle, username: &str) {
    item_ctx.add_copied_node(ui);
    item_ctx.set_text_by_id("username", username);
}

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