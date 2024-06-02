use std::collections::BTreeMap;

use bevy_ecs::{system::{Resource, Query}, event::EventReader, entity::Entity};

use game_engine::{ui::{NodeActiveState, UiManager}, session::{channels, messages, SessionClient, components::GlobalChatMessage}, input::{InputEvent, Key}, social::GlobalChatMessageId, ui::{extensions::ListUiExt, UiHandle}};

use crate::ui::{go_to_sub_ui, UiCatalog, UiKey};

#[derive(Resource)]
pub struct GlobalChat {
    global_chats: BTreeMap<GlobalChatMessageId, Entity>,
    list_ui_ext: ListUiExt,
}

impl Default for GlobalChat {
    fn default() -> Self {
        Self {
            global_chats: BTreeMap::new(),
            list_ui_ext: ListUiExt::new(),
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

        global_chat_messages.set_list_ui_container(ui_manager, message_q, &ui_handle, container_id_str);
    }

    pub(crate) fn on_load_list_item_ui(
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        message_q: &Query<&GlobalChatMessage>,
        global_chat_messages: &mut GlobalChat,
    ) {
        let item_ui_key = UiKey::GlobalChatListItem;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        global_chat_messages.set_list_item_ui(ui_manager, message_q, &item_ui_handle);
    }

    pub(crate) fn set_list_ui_container(
        &mut self,
        ui_manager: &mut UiManager,
        message_q: &Query<&GlobalChatMessage>,
        ui_handle: &UiHandle,
        id_str: &str
    ) {
        self.list_ui_ext.set_container_ui(ui_manager, ui_handle, id_str);
        self.sync_with_collection(ui_manager, message_q);
    }

    pub(crate) fn set_list_item_ui(
        &mut self,
        ui_manager: &mut UiManager,
        message_q: &Query<&GlobalChatMessage>,
        ui_handle: &UiHandle
    ) {
        self.list_ui_ext.set_item_ui(ui_manager, ui_handle);
        self.sync_with_collection(ui_manager, message_q);
    }

    pub fn add_message(
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
        self.list_ui_ext.sync_with_collection(
            ui_manager,
            &self.global_chats,
            |ui_runtime, id_str_to_node_map, _message_id, message_entity| {
                let message = message_q.get(*message_entity).unwrap();

                let message_user_id: u64 = (*message.user_id).into();
                let message_user_id_node_id = id_str_to_node_map.get("user_name").unwrap();
                ui_runtime.set_text(message_user_id_node_id, message_user_id.to_string().as_str());

                let message_timestamp = message.timestamp.to_string();
                let message_timestamp_node_id = id_str_to_node_map.get("timestamp").unwrap();
                ui_runtime.set_text(message_timestamp_node_id, message_timestamp.as_str());

                let message_text = message.message.as_str();
                let message_text_node_id = id_str_to_node_map.get("message").unwrap();
                ui_runtime.set_text(message_text_node_id, message_text);
            },
        );
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