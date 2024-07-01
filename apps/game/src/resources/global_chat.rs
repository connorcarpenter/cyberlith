use std::collections::BTreeMap;

use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    system::{Query, Resource},
};

use game_engine::{
    asset::AssetManager,
    input::{InputEvent, Key},
    logging::info,
    session::{channels, components::GlobalChatMessage, messages, SessionClient},
    social::GlobalChatMessageId,
    ui::{
        extensions::{ListUiExt, ListUiExtItem},
        NodeActiveState, UiHandle, UiManager,
    },
};

use crate::ui::{go_to_sub_ui, UiCatalog, UiKey};

#[derive(Resource)]
pub struct GlobalChat {
    global_chats: BTreeMap<GlobalChatMessageId, Entity>,
    list_ui_ext: ListUiExt<GlobalChatMessageId>,
    message_item_ui: Option<UiHandle>,
    username_and_message_item_ui: Option<UiHandle>,
    day_divider_item_ui: Option<UiHandle>,
}

impl Default for GlobalChat {
    fn default() -> Self {
        Self {
            global_chats: BTreeMap::new(),
            list_ui_ext: ListUiExt::new(false),
            message_item_ui: None,
            username_and_message_item_ui: None,
            day_divider_item_ui: None,
        }
    }
}

impl GlobalChat {
    pub(crate) fn handle_events(
        global_chat: &mut GlobalChat,
        ui_manager: &mut UiManager,
        ui_catalog: &UiCatalog,
        asset_manager: &AssetManager,
        session_server: &mut SessionClient,
        input_events: &mut EventReader<InputEvent>,
        message_q: &Query<&GlobalChatMessage>,
        _should_rumble: &mut bool,
    ) {
        let ui_handle = ui_catalog.get_ui_handle(UiKey::GlobalChat);

        for event in input_events.read() {
            match event {
                // TODO this probably doesn't belong here! this is where it is required to be selecting the textbox!!!
                InputEvent::KeyPressed(Key::I, _) => {
                    info!("I Key Pressed");
                    if let Some(NodeActiveState::Active) =
                        ui_manager.get_node_active_state_from_id(&ui_handle, "message_textbox")
                    {
                        // do nothing, typing
                        info!("Node Is Active");
                    } else {
                        info!("Scrolling Up");
                        global_chat.list_ui_ext.scroll_up();
                        global_chat.sync_with_collection(ui_manager, asset_manager, message_q);
                    }
                }
                InputEvent::KeyPressed(Key::J, _) => {
                    info!("J Key Pressed");
                    if let Some(NodeActiveState::Active) =
                        ui_manager.get_node_active_state_from_id(&ui_handle, "message_textbox")
                    {
                        // do nothing, typing
                        info!("Node Is Active");
                    } else {
                        info!("Scrolling Down");
                        global_chat.list_ui_ext.scroll_down();
                        global_chat.sync_with_collection(ui_manager, asset_manager, message_q);
                    }
                }
                InputEvent::KeyPressed(Key::Enter, modifiers) => {
                    if let Some(NodeActiveState::Active) =
                        ui_manager.get_node_active_state_from_id(&ui_handle, "message_textbox")
                    {
                        if modifiers.shift {
                            // later, add multi-line newline
                        } else {
                            // send message
                            GlobalChat::send_message(ui_manager, &ui_handle, session_server)
                        }
                    };
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
        asset_manager: &AssetManager,
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

        global_chat_messages
            .list_ui_ext
            .set_container_ui(ui_manager, &ui_handle, container_id_str);
        global_chat_messages.sync_with_collection(ui_manager, asset_manager, message_q);
    }

    pub(crate) fn on_load_day_divider_item_ui(
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        message_q: &Query<&GlobalChatMessage>,
        global_chat_messages: &mut GlobalChat,
    ) {
        let item_ui_key = UiKey::GlobalChatDayDivider;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        global_chat_messages.day_divider_item_ui = Some(item_ui_handle.clone());
        global_chat_messages.sync_with_collection(ui_manager, asset_manager, message_q);
    }

    pub(crate) fn on_load_username_and_message_item_ui(
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        message_q: &Query<&GlobalChatMessage>,
        global_chat_messages: &mut GlobalChat,
    ) {
        let item_ui_key = UiKey::GlobalChatUsernameAndMessage;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        global_chat_messages.username_and_message_item_ui = Some(item_ui_handle.clone());
        global_chat_messages.sync_with_collection(ui_manager, asset_manager, message_q);
    }

    pub(crate) fn on_load_message_item_ui(
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        message_q: &Query<&GlobalChatMessage>,
        global_chat_messages: &mut GlobalChat,
    ) {
        let item_ui_key = UiKey::GlobalChatMessage;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        global_chat_messages.message_item_ui = Some(item_ui_handle.clone());
        global_chat_messages.sync_with_collection(ui_manager, asset_manager, message_q);
    }

    pub fn recv_message(
        &mut self,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        message_q: &Query<&GlobalChatMessage>,
        message_id: GlobalChatMessageId,
        message_entity: Entity,
    ) {
        self.global_chats.insert(message_id, message_entity);

        if self.global_chats.len() > 100 {
            self.global_chats.pop_first();
        }

        self.sync_with_collection(ui_manager, asset_manager, message_q);
    }

    pub fn sync_with_collection(
        &mut self,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        message_q: &Query<&GlobalChatMessage>,
    ) {
        if self.message_item_ui.is_none()
            || self.day_divider_item_ui.is_none()
            || self.username_and_message_item_ui.is_none()
        {
            return;
        }

        let day_divider_ui_handle = self.day_divider_item_ui.as_ref().unwrap();
        let username_and_message_ui_handle = self.username_and_message_item_ui.as_ref().unwrap();
        let message_ui_handle = self.message_item_ui.as_ref().unwrap();

        self.list_ui_ext.sync_with_collection(
            ui_manager,
            asset_manager,
            self.global_chats.iter(),
            self.global_chats.len(),
            |item_ctx, message_id, prev_message_id_opt| {
                let message_entity = *(self.global_chats.get(&message_id).unwrap());
                let message = message_q.get(message_entity).unwrap();
                let message_timestamp = (*message.timestamp).clone();
                let message_user_id = *message.user_id;

                let (prev_timestamp_opt, prev_user_id_opt) = match prev_message_id_opt {
                    Some(prev_message_id) => {
                        let prev_message_entity =
                            *(self.global_chats.get(&prev_message_id).unwrap());
                        let prev_message = message_q.get(prev_message_entity).unwrap();
                        let prev_timestamp = (*prev_message.timestamp).clone();
                        let prev_message_user_id = *prev_message.user_id;
                        (Some(prev_timestamp), Some(prev_message_user_id))
                    }
                    None => (None, None),
                };

                let mut added_divider = false;

                // add day divider if necessary
                if prev_timestamp_opt.is_none() || prev_timestamp_opt.unwrap() != message_timestamp
                {
                    Self::add_day_divider_item(item_ctx, day_divider_ui_handle, message);
                    added_divider = true;
                }

                // add username if necessary
                if prev_user_id_opt.is_none() || added_divider {
                    Self::add_username_and_message_item(
                        item_ctx,
                        username_and_message_ui_handle,
                        message,
                    );
                } else if prev_user_id_opt.unwrap() != message_user_id {
                    Self::add_message_item(item_ctx, message_ui_handle, " "); // blank space
                    Self::add_username_and_message_item(
                        item_ctx,
                        username_and_message_ui_handle,
                        message,
                    );
                } else {
                    // just add message
                    Self::add_message_item(item_ctx, message_ui_handle, message.message.as_str());
                }
            },
        );
    }

    fn add_day_divider_item(
        item_ctx: &mut ListUiExtItem<GlobalChatMessageId>,
        ui: &UiHandle,
        message: &GlobalChatMessage,
    ) {
        item_ctx.add_copied_node(ui);

        let divider_date_str = message.timestamp.date_string();
        item_ctx.set_text_by_str("timestamp", divider_date_str.as_str());
    }

    fn add_username_and_message_item(
        item_ctx: &mut ListUiExtItem<GlobalChatMessageId>,
        ui: &UiHandle,
        message: &GlobalChatMessage,
    ) {
        item_ctx.add_copied_node(ui);

        let message_user_id: u64 = (*message.user_id).into();
        item_ctx.set_text_by_str("user_name", message_user_id.to_string().as_str());

        let message_timestamp = message.timestamp.time_string();
        item_ctx.set_text_by_str("timestamp", message_timestamp.as_str());

        let message_text = message.message.as_str();
        item_ctx.set_text_by_str("message", message_text);
    }

    fn add_message_item(
        item_ctx: &mut ListUiExtItem<GlobalChatMessageId>,
        ui: &UiHandle,
        message_text: &str,
    ) {
        item_ctx.add_copied_node(ui);

        item_ctx.set_text_by_str("message", message_text);
    }

    fn send_message(
        ui_manager: &mut UiManager,
        ui_handle: &UiHandle,
        session_server: &mut SessionClient,
    ) {
        let Some(textbox_text) = ui_manager.get_textbox_text(ui_handle, "message_textbox") else {
            return;
        };

        ui_manager.set_textbox_text(ui_handle, "message_textbox", "");

        // info!("Sending message: {:?}", textbox_text);

        let message = messages::GlobalChatSendMessage::new(&textbox_text);
        session_server.send_message::<channels::ClientActionsChannel, _>(&message);
    }
}
