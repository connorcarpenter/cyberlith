use std::collections::{BTreeMap, HashMap};

use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    system::{Query, Resource},
};

use game_engine::{
    asset::AssetManager,
    input::{InputEvent, Key},
    logging::{info, warn},
    session::{
        channels,
        components::{ChatMessage, User},
        messages, SessionClient,
    },
    social::{LobbyId, MessageId},
    ui::{
        extensions::{ListUiExt, ListUiExtItem},
        NodeActiveState, UiHandle, UiManager,
    },
};

use crate::ui::{events::ResyncMessageListUiEvent, go_to_sub_ui, UiCatalog, UiKey};

#[derive(Resource)]
pub struct ChatMessageManager {
    messages: HashMap<Option<LobbyId>, BTreeMap<MessageId, Entity>>,

    list_ui_ext: ListUiExt<MessageId>,
    message_item_ui: Option<UiHandle>,
    username_and_message_item_ui: Option<UiHandle>,
    day_divider_item_ui: Option<UiHandle>,
}

impl Default for ChatMessageManager {
    fn default() -> Self {
        let mut messages = HashMap::new();
        messages.insert(None, BTreeMap::new());

        Self {
            messages,
            list_ui_ext: ListUiExt::new(false),
            message_item_ui: None,
            username_and_message_item_ui: None,
            day_divider_item_ui: None,
        }
    }
}

impl ChatMessageManager {
    pub(crate) fn handle_events(
        &mut self,
        ui_manager: &mut UiManager,
        ui_catalog: &UiCatalog,
        asset_manager: &AssetManager,
        session_client: &mut SessionClient,
        input_events: &mut EventReader<InputEvent>,
        resync_chat_message_ui_events: &mut EventReader<ResyncMessageListUiEvent>,
        user_q: &Query<&User>,
        chat_message_q: &Query<&ChatMessage>,
        _should_rumble: &mut bool,
    ) {
        let ui_handle = ui_catalog.get_ui_handle(UiKey::GlobalChat);

        let mut should_resync = None;
        for resync_event in resync_chat_message_ui_events.read() {
            if resync_event.maintain_scroll() {
                if should_resync.is_none() {
                    should_resync = Some(true);
                }
            } else {
                should_resync = Some(false);
            }
        }

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
                        self.list_ui_ext.scroll_up();
                        should_resync = Some(false);
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
                        self.list_ui_ext.scroll_down();
                        should_resync = Some(false);
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
                            Self::send_message(ui_manager, &ui_handle, session_client)
                        }
                    };
                }
                _ => {}
            }
        }

        if let Some(maintain_scroll) = should_resync {
            let is_bottom_visible = self.list_ui_ext.is_bottom_visible();

            self.sync_with_collection(
                session_client,
                ui_manager,
                asset_manager,
                user_q,
                chat_message_q,
            );

            if is_bottom_visible && maintain_scroll {
                self.list_ui_ext.scroll_to_bottom();
                self.sync_with_collection(
                    session_client,
                    ui_manager,
                    asset_manager,
                    user_q,
                    chat_message_q,
                );
            }
        }
    }

    pub(crate) fn on_load_container_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        resync_chat_message_ui_events: &mut EventWriter<ResyncMessageListUiEvent>,
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

        // setup global chat list extension
        {
            let container_id_str = "chat_wall";

            self.list_ui_ext
                .set_container_ui(ui_manager, &ui_handle, container_id_str);
            resync_chat_message_ui_events.send(ResyncMessageListUiEvent::new(true));
        }
    }

    pub(crate) fn on_load_day_divider_item_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        resync_chat_message_ui_events: &mut EventWriter<ResyncMessageListUiEvent>,
    ) {
        let item_ui_key = UiKey::GlobalChatDayDivider;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        self.day_divider_item_ui = Some(item_ui_handle.clone());
        resync_chat_message_ui_events.send(ResyncMessageListUiEvent::new(true));
    }

    pub(crate) fn on_load_username_and_message_item_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        resync_global_chat_events: &mut EventWriter<ResyncMessageListUiEvent>,
    ) {
        let item_ui_key = UiKey::GlobalChatUsernameAndMessage;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        self.username_and_message_item_ui = Some(item_ui_handle.clone());
        resync_global_chat_events.send(ResyncMessageListUiEvent::new(true));
    }

    pub(crate) fn on_load_message_item_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        resync_global_chat_events: &mut EventWriter<ResyncMessageListUiEvent>,
    ) {
        let item_ui_key = UiKey::GlobalChatMessage;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        self.message_item_ui = Some(item_ui_handle.clone());
        resync_global_chat_events.send(ResyncMessageListUiEvent::new(true));
    }

    pub fn recv_message(
        &mut self,
        lobby_id_opt: &Option<LobbyId>,
        resync_lobby_global_events: &mut EventWriter<ResyncMessageListUiEvent>,
        message_id: MessageId,
        message_entity: Entity,
    ) {
        if !self.messages.contains_key(lobby_id_opt) {
            self.messages.insert(lobby_id_opt.clone(), BTreeMap::new());
        }
        let lobby_messages = self.messages.get_mut(lobby_id_opt).unwrap();
        lobby_messages.insert(message_id, message_entity);

        if lobby_messages.len() > 100 {
            lobby_messages.pop_first();
        }

        resync_lobby_global_events.send(ResyncMessageListUiEvent::new(true));
    }

    pub fn sync_with_collection(
        &mut self,
        session_client: &SessionClient,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        user_q: &Query<&User>,
        message_q: &Query<&ChatMessage>,
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

        let messages = self.messages.get_mut(&None).unwrap();

        self.list_ui_ext.sync_with_collection(
            ui_manager,
            asset_manager,
            messages.iter(),
            messages.len(),
            |item_ctx, message_id, prev_message_id_opt| {
                let message_entity = *(messages.get(&message_id).unwrap());
                let message = message_q.get(message_entity).unwrap();
                let message_timestamp = (*message.timestamp).clone();
                let message_user_entity = message.owner_user_entity.get(session_client).unwrap();

                let (prev_timestamp_opt, prev_message_user_entity) = match prev_message_id_opt {
                    Some(prev_message_id) => {
                        let prev_message_entity = *(messages.get(&prev_message_id).unwrap());
                        let prev_message = message_q.get(prev_message_entity).unwrap();
                        let prev_timestamp = (*prev_message.timestamp).clone();
                        let prev_message_user_entity =
                            prev_message.owner_user_entity.get(session_client).unwrap();
                        (Some(prev_timestamp), Some(prev_message_user_entity))
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
                if prev_message_user_entity.is_none() || added_divider {
                    Self::add_username_and_message_item(
                        session_client,
                        user_q,
                        item_ctx,
                        username_and_message_ui_handle,
                        message,
                    );
                } else if prev_message_user_entity.unwrap() != message_user_entity {
                    Self::add_message_item(item_ctx, message_ui_handle, " "); // blank space
                    Self::add_username_and_message_item(
                        session_client,
                        user_q,
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
        item_ctx: &mut ListUiExtItem<MessageId>,
        ui: &UiHandle,
        message: &ChatMessage,
    ) {
        item_ctx.add_copied_node(ui);

        let divider_date_str = message.timestamp.date_string();
        item_ctx.set_text_by_id("timestamp", divider_date_str.as_str());
    }

    fn add_username_and_message_item(
        session_client: &SessionClient,
        user_q: &Query<&User>,
        item_ctx: &mut ListUiExtItem<MessageId>,
        ui: &UiHandle,
        message: &ChatMessage,
    ) {
        let Some(user_info_entity) = message.owner_user_entity.get(session_client) else {
            warn!("User info not found for Message: {:?}", *message.id);
            return;
        };

        let message_user_name = {
            if let Ok(user_public_info) = user_q.get(user_info_entity) {
                user_public_info.name.as_str().to_string()
            } else {
                "?".to_string()
            }
        };

        item_ctx.add_copied_node(ui);

        item_ctx.set_text_by_id("user_name", message_user_name.as_str());

        let message_timestamp = message.timestamp.time_string();
        item_ctx.set_text_by_id("timestamp", message_timestamp.as_str());

        let message_text = message.message.as_str();
        item_ctx.set_text_by_id("message", message_text);
    }

    fn add_message_item(
        item_ctx: &mut ListUiExtItem<MessageId>,
        ui: &UiHandle,
        message_text: &str,
    ) {
        item_ctx.add_copied_node(ui);

        item_ctx.set_text_by_id("message", message_text);
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
