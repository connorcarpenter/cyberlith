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
pub struct GlobalChatState {
    pub global_chat_list_ui_ext: ListUiExt<u32>,
    pub global_chats: BTreeMap<u32, (String, u8, u8, u8, u8, String)>,

    day_divider_ui_handle: UiHandle,
    username_and_message_ui_handle: UiHandle,
    message_ui_handle: UiHandle,
}

impl GlobalChatState {
    pub fn new(
        day_divider_ui_handle: &UiHandle,
        username_and_message_ui_handle: &UiHandle,
        message_ui_handle: &UiHandle,
    ) -> Self {
        Self {
            global_chat_list_ui_ext: ListUiExt::new(false),
            global_chats: BTreeMap::new(),

            day_divider_ui_handle: day_divider_ui_handle.clone(),
            username_and_message_ui_handle: username_and_message_ui_handle.clone(),
            message_ui_handle: message_ui_handle.clone(),
        }
    }

    pub fn global_chat_scroll_up(&mut self, ui_manager: &mut UiManager, asset_manager: &AssetManager) {
        self.global_chat_list_ui_ext.scroll_up();

        self.sync_chat_collections(ui_manager, asset_manager);
    }

    pub fn global_chat_scroll_down(&mut self, ui_manager: &mut UiManager, asset_manager: &AssetManager) {
        self.global_chat_list_ui_ext.scroll_down();

        self.sync_chat_collections(ui_manager, asset_manager);
    }

    pub fn sync_chat_collections(
        &mut self,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
    ) {
        // setup collection
        self.global_chat_list_ui_ext.sync_with_collection(
            ui_manager,
            asset_manager,
            self.global_chats.iter(),
            self.global_chats.len(),
            |item_ctx, message_id, prev_message_id_opt| {
                // info!("syncing chat message: {} {} {} {} {} {}", username, month, day, hour, minute, message);
                let (username, month, day, hour, minute, message) =
                    self.global_chats.get(&message_id).unwrap();

                let (prev_date, prev_username) = match prev_message_id_opt {
                    Some(prev_message_id) => {
                        let (prev_username, prev_month, prev_day, _, _, _) =
                            self.global_chats.get(&prev_message_id).unwrap();
                        (Some((*prev_month, *prev_day)), Some(prev_username.clone()))
                    }
                    None => (None, None),
                };

                let message_date = (*month, *day);
                let message_time = (*hour, *minute);
                let mut added_divider = false;

                // add day divider if necessary
                if prev_date.is_none() || prev_date.unwrap() != message_date {
                    add_day_divider_item(item_ctx, &self.day_divider_ui_handle, message_date);

                    added_divider = true;
                }

                // add username if necessary
                if prev_username.is_none() || added_divider {
                    add_username_and_message_item(
                        item_ctx,
                        &self.username_and_message_ui_handle,
                        username,
                        message_time,
                        message,
                    );
                } else if !prev_username.as_ref().unwrap().eq(username) {
                    add_message_item(item_ctx, &self.message_ui_handle, " "); // blank space
                    add_username_and_message_item(
                        item_ctx,
                        &self.username_and_message_ui_handle,
                        username,
                        message_time,
                        message,
                    );
                } else {
                    // just add message
                    add_message_item(item_ctx, &self.message_ui_handle, message);
                }
            },
        );
    }
}

fn add_day_divider_item(item_ctx: &mut ListUiExtItem<u32>, ui: &UiHandle, date: (u8, u8)) {
    item_ctx.add_copied_node(ui);

    let divider_date_str = format!("{}/{}", date.0, date.1);
    item_ctx.set_text_by_id("timestamp", divider_date_str.as_str());
}

fn add_username_and_message_item(
    item_ctx: &mut ListUiExtItem<u32>,
    ui: &UiHandle,
    username: &str,
    time: (u8, u8),
    message_text: &str,
) {
    item_ctx.add_copied_node(ui);

    item_ctx.set_text_by_id("user_name", username);

    let divider_date_str = format!("{}:{}", time.0, time.1);
    item_ctx.set_text_by_id("timestamp", divider_date_str.as_str());

    item_ctx.set_text_by_id("message", message_text);
}

fn add_message_item(item_ctx: &mut ListUiExtItem<u32>, ui: &UiHandle, message_text: &str) {
    item_ctx.add_copied_node(ui);
    item_ctx.set_text_by_id("message", message_text);
}