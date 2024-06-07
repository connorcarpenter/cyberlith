use std::collections::BTreeMap;

use bevy_ecs::{entity::Entity, system::Resource};

use game_engine::ui::{UiHandle, extensions::{ListUiExt, ListUiExtItem}, UiManager};
use logging::info;

#[derive(Resource)]
pub struct Global {
    pub scene_camera_entity: Entity,
    pub list_ui_ext: ListUiExt<u32>,
    pub global_chats: BTreeMap<u32, (String, u8, u8, u8, u8, String)>,
    pub ui_handles: Vec<UiHandle>,
}

impl Global {
    pub fn new(scene_camera_entity: Entity) -> Self {
        Self {
            scene_camera_entity,
            list_ui_ext: ListUiExt::new(false),
            global_chats: BTreeMap::new(),
            ui_handles: Vec::new(),
        }
    }

    pub fn scroll_up(&mut self, ui_manager: &mut UiManager) {

        self.list_ui_ext.scroll_up();

        self.sync_chat_collections(ui_manager);
    }

    pub fn scroll_down(&mut self, ui_manager: &mut UiManager) {

        self.list_ui_ext.scroll_down();

        self.sync_chat_collections(ui_manager);
    }

    pub fn sync_chat_collections(
        &mut self,
        ui_manager: &mut UiManager,
    ) {
        // day divider ui
        let day_divider_ui_handle = self.ui_handles[2];
        let username_and_message_ui_handle = self.ui_handles[3];
        let message_ui_handle = self.ui_handles[4];

        let mut last_date: Option<(u8, u8)> = None;
        let mut last_username: Option<String> = None;

        // setup collection
        self.list_ui_ext.sync_with_collection(
            ui_manager,
            self.global_chats.iter(),
            self.global_chats.len(),
            |
                item_ctx,
                _message_id,
                (
                    username,
                    month,
                    day,
                    hour,
                    minute,
                    message
                )
            | {
                info!("syncing chat message: {} {} {} {} {} {}", username, month, day, hour, minute, message);

                let message_date = (*month, *day);
                let message_time = (*hour, *minute);

                // add day divider if necessary
                if last_date.is_none() || last_date.unwrap() != message_date {
                    add_day_divider_item(item_ctx, &day_divider_ui_handle, message_date);

                    last_username = None;
                }

                last_date = Some(message_date);

                // add username if necessary
                if last_username.is_none() {
                    add_username_and_message_item(item_ctx, &username_and_message_ui_handle, username, message_time, message);
                } else if !last_username.as_ref().unwrap().eq(username) {
                    add_message_item(item_ctx, &message_ui_handle, " "); // blank space
                    add_username_and_message_item(item_ctx, &username_and_message_ui_handle, username, message_time, message);
                } else {
                    // just add message
                    add_message_item(item_ctx, &message_ui_handle, message);
                }

                last_username = Some(username.clone());
            },
        );
    }
}

fn add_day_divider_item(item_ctx: &mut ListUiExtItem<u32>, ui: &UiHandle, date: (u8, u8)) {

    item_ctx.add_copied_node(ui);

    let divider_date_str = format!("{}/{}", date.0, date.1);
    item_ctx.set_text_by_str("timestamp", divider_date_str.as_str());
}

fn add_username_and_message_item(item_ctx: &mut ListUiExtItem<u32>, ui: &UiHandle, username: &str, time: (u8, u8), message_text: &str) {

    item_ctx.add_copied_node(ui);

    item_ctx.set_text_by_str("user_name", username);

    let divider_date_str = format!("{}:{}", time.0, time.1);
    item_ctx.set_text_by_str("timestamp", divider_date_str.as_str());

    item_ctx.set_text_by_str("message", message_text);
}

fn add_message_item(item_ctx: &mut ListUiExtItem<u32>, ui: &UiHandle, message_text: &str) {
    item_ctx.add_copied_node(ui);
    item_ctx.set_text_by_str("message", message_text);
}