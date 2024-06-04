use std::collections::BTreeMap;

use bevy_ecs::{system::{Resource, Query}, event::EventReader, entity::Entity};

use game_engine::{auth::UserId, ui::{NodeActiveState, UiManager}, session::{channels, messages, SessionClient, components::GlobalChatMessage}, input::{InputEvent, Key}, social::GlobalChatMessageId, ui::{extensions::{ListUiExt, ListUiExtItem}, UiHandle}};

use crate::ui::{go_to_sub_ui, UiCatalog, UiKey};

#[derive(Resource)]
pub struct GlobalChat {
    global_chats: BTreeMap<GlobalChatMessageId, Entity>,
    list_ui_ext: ListUiExt,
    message_item_ui: Option<UiHandle>,
    username_and_message_item_ui: Option<UiHandle>,
    day_divider_item_ui: Option<UiHandle>,
}

impl Default for GlobalChat {
    fn default() -> Self {
        Self {
            global_chats: BTreeMap::new(),
            list_ui_ext: ListUiExt::new(),
            message_item_ui: None,
            username_and_message_item_ui: None,
            day_divider_item_ui: None,
        }
    }
}

impl GlobalChat {

    pub(crate) fn handle_events(
        ui_manager: &mut UiManager,
        ui_catalog: &UiCatalog,
        session_server: &mut SessionClient,
        input_events: &mut EventReader<InputEvent>,
        _should_rumble: &mut bool,
    ) {
        let ui_handle = ui_catalog.get_ui_handle(UiKey::GlobalChat);
        let Some(NodeActiveState::Active) = ui_manager.get_node_active_state_from_id(&ui_handle, "message_textbox") else {
            return;
        };

        for event in input_events.read() {
            match event {
                InputEvent::KeyPressed(Key::Enter, modifiers) => {
                    if modifiers.shift {
                        // later, add multi-line newline
                    } else {
                        // send message
                        GlobalChat::send_message(ui_manager, &ui_handle, session_server)
                    }
                }
                _ => {}
            }
        }
    }

    pub fn reset_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
        // TODO: implement
    }

    pub(crate) fn on_load_container_ui(
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        message_q: &Query<&GlobalChatMessage>,
        global_chat_messages: &mut GlobalChat,
    ) {
        let ui_key = UiKey::GlobalChat;
        let ui_handle = ui_catalog.get_ui_handle(ui_key);

        ui_catalog.set_loaded(ui_key);

        // set sub-ui to GlobalChat at beginning
        if let Some(active_ui_handle) = ui_manager.active_ui() {
            if ui_catalog.get_ui_key(&active_ui_handle) == UiKey::MainMenu {
                go_to_sub_ui(ui_manager, ui_catalog, UiKey::GlobalChat);
            }
        }

        // setup list extension
        let container_id_str = "chat_wall";

        global_chat_messages.list_ui_ext.set_container_ui(ui_manager, &ui_handle, container_id_str);
        global_chat_messages.sync_with_collection(ui_manager, message_q);
    }

    pub(crate) fn on_load_day_divider_item_ui(
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        message_q: &Query<&GlobalChatMessage>,
        global_chat_messages: &mut GlobalChat,
    ) {
        let item_ui_key = UiKey::GlobalChatDayDivider;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        global_chat_messages.day_divider_item_ui = Some(item_ui_handle.clone());
        global_chat_messages.sync_with_collection(ui_manager, message_q);
    }

    pub(crate) fn on_load_username_and_message_item_ui(
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        message_q: &Query<&GlobalChatMessage>,
        global_chat_messages: &mut GlobalChat,
    ) {
        let item_ui_key = UiKey::GlobalChatUsernameAndMessage;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        global_chat_messages.username_and_message_item_ui = Some(item_ui_handle.clone());
        global_chat_messages.sync_with_collection(ui_manager, message_q);
    }

    pub(crate) fn on_load_message_item_ui(
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        message_q: &Query<&GlobalChatMessage>,
        global_chat_messages: &mut GlobalChat,
    ) {
        let item_ui_key = UiKey::GlobalChatMessage;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        global_chat_messages.message_item_ui = Some(item_ui_handle.clone());
        global_chat_messages.sync_with_collection(ui_manager, message_q);
    }

    pub fn recv_message(
        &mut self,
        ui_manager: &mut UiManager,
        message_q: &Query<&GlobalChatMessage>,
        message_id: GlobalChatMessageId,
        message_entity: Entity
    ) {

        self.global_chats.insert(message_id, message_entity);

        if self.global_chats.len() > 100 {
            self.global_chats.pop_first();
        }

        self.sync_with_collection(ui_manager, message_q);
    }

    fn sync_with_collection(
        &mut self,
        ui_manager: &mut UiManager,
        message_q: &Query<&GlobalChatMessage>,
    ) {
        if self.message_item_ui.is_none() {
            return;
        }

        let day_divider_ui_handle = self.day_divider_item_ui.as_ref().unwrap();
        let username_and_message_ui_handle = self.username_and_message_item_ui.as_ref().unwrap();
        let message_ui_handle = self.message_item_ui.as_ref().unwrap();
        let mut last_date: Option<(u8, u8, u16)> = None;
        let mut last_user_id: Option<UserId> = None;

        self.list_ui_ext.sync_with_collection(
            ui_manager,
            &self.global_chats,
            |item_ctx, _message_id, message_entity| {
                let message = message_q.get(*message_entity).unwrap();

                let message_date = message.timestamp.date();
                let message_user_id = *message.user_id;

                // add day divider if necessary
                if last_date.is_none() || last_date.unwrap() != message_date {
                    Self::add_day_divider_item(item_ctx, day_divider_ui_handle, message);
                    last_user_id = None;
                }

                last_date = Some(message_date);

                // add username if necessary
                if last_user_id.is_none() || last_user_id.unwrap() != message_user_id {
                    Self::add_username_and_message_item(item_ctx, username_and_message_ui_handle, message);
                } else {

                    // just add message
                    Self::add_message_item(item_ctx, message_ui_handle, message);
                }

                last_user_id = Some(message_user_id);
            },
        );
    }

    fn add_day_divider_item(item_ctx: &mut ListUiExtItem, ui: &UiHandle, message: &GlobalChatMessage) {

        item_ctx.add_copied_node(ui);

        let divider_date_str = message.timestamp.date_string();
        let divider_text_node_id = item_ctx.get_node_id_by_str("time").unwrap();
        item_ctx.set_text(&divider_text_node_id, divider_date_str.as_str());
    }

    fn add_username_and_message_item(item_ctx: &mut ListUiExtItem, ui: &UiHandle, message: &GlobalChatMessage) {

        item_ctx.add_copied_node(ui);

        let message_user_id: u64 = (*message.user_id).into();
        let message_user_id_node_id = item_ctx.get_node_id_by_str("user_name").unwrap();
        item_ctx.set_text(&message_user_id_node_id, message_user_id.to_string().as_str());

        let message_timestamp = message.timestamp.time_string();
        let message_timestamp_node_id = item_ctx.get_node_id_by_str("timestamp").unwrap();
        item_ctx.set_text(&message_timestamp_node_id, message_timestamp.as_str());

        let message_text = message.message.as_str();
        let message_text_node_id = item_ctx.get_node_id_by_str("message").unwrap();
        item_ctx.set_text(&message_text_node_id, message_text);
    }

    fn add_message_item(item_ctx: &mut ListUiExtItem, ui: &UiHandle, message: &GlobalChatMessage) {

        item_ctx.add_copied_node(ui);

        let message_text = message.message.as_str();
        let message_text_node_id = item_ctx.get_node_id_by_str("message").unwrap();
        item_ctx.set_text(&message_text_node_id, message_text);
    }

    fn send_message(
        ui_manager: &mut UiManager,
        ui_handle: &UiHandle,
        session_server: &mut SessionClient,
    ) {
        let Some(textbox_text) = ui_manager.get_textbox_text(
            ui_handle,
            "message_textbox"
        ) else {
            return;
        };

        ui_manager.set_textbox_text(
            ui_handle,
            "message_textbox",
            ""
        );

        // info!("Sending message: {:?}", textbox_text);

        let message = messages::GlobalChatSendMessage::new(&textbox_text);
        session_server.send_message::<channels::ClientActionsChannel, _>(&message);
    }
}