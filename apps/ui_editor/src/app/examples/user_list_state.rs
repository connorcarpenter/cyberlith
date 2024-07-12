use std::collections::BTreeMap;

use bevy_ecs::{system::Resource};

use game_engine::{
    asset::AssetManager,
    ui::{
        extensions::{ListUiExt, ListUiExtItem},
        UiHandle, UiManager,
    },
};

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