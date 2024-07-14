use std::collections::BTreeMap;

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
        components::{MessagePublic, UserPublic},
        messages, SessionClient,
    },
    social::MessageId,
    ui::{
        extensions::{ListUiExt, ListUiExtItem},
        NodeActiveState, UiHandle, UiManager,
    },
};

use crate::ui::{events::ResyncGlobalChatEvent, go_to_sub_ui, UiCatalog, UiKey};

#[derive(Resource)]
pub struct GlobalChat {
    global_chats: BTreeMap<MessageId, Entity>,
    list_ui_ext: ListUiExt<MessageId>,
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
        &mut self,
        ui_manager: &mut UiManager,
        ui_catalog: &UiCatalog,
        asset_manager: &AssetManager,
        session_server: &mut SessionClient,
        input_events: &mut EventReader<InputEvent>,
        resync_global_chat_events: &mut EventReader<ResyncGlobalChatEvent>,
        user_q: &Query<&UserPublic>,
        message_q: &Query<&MessagePublic>,
        _should_rumble: &mut bool,
    ) {
        let ui_handle = ui_catalog.get_ui_handle(UiKey::GlobalChat);

        let mut should_resync = None;
        for resync_event in resync_global_chat_events.read() {
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
                            Self::send_message(ui_manager, &ui_handle, session_server)
                        }
                    };
                }
                _ => {}
            }
        }

        if let Some(maintain_scroll) = should_resync {
            let is_bottom_visible = self.list_ui_ext.is_bottom_visible();

            self.sync_with_collection(session_server, ui_manager, asset_manager, user_q, message_q);

            if is_bottom_visible && maintain_scroll {
                self.list_ui_ext.scroll_to_bottom();
                self.sync_with_collection(
                    session_server,
                    ui_manager,
                    asset_manager,
                    user_q,
                    message_q,
                );
            }
        }
    }

    pub fn reset_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
        // TODO: implement
    }

    pub(crate) fn on_load_container_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        resync_global_chat_events: &mut EventWriter<ResyncGlobalChatEvent>,
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
            resync_global_chat_events.send(ResyncGlobalChatEvent::new(true));
        }
    }

    pub(crate) fn on_load_day_divider_item_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        resync_global_chat_events: &mut EventWriter<ResyncGlobalChatEvent>,
    ) {
        let item_ui_key = UiKey::GlobalChatDayDivider;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        self.day_divider_item_ui = Some(item_ui_handle.clone());
        resync_global_chat_events.send(ResyncGlobalChatEvent::new(true));
    }

    pub(crate) fn on_load_username_and_message_item_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        resync_global_chat_events: &mut EventWriter<ResyncGlobalChatEvent>,
    ) {
        let item_ui_key = UiKey::GlobalChatUsernameAndMessage;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        self.username_and_message_item_ui = Some(item_ui_handle.clone());
        resync_global_chat_events.send(ResyncGlobalChatEvent::new(true));
    }

    pub(crate) fn on_load_message_item_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        resync_global_chat_events: &mut EventWriter<ResyncGlobalChatEvent>,
    ) {
        let item_ui_key = UiKey::GlobalChatMessage;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        self.message_item_ui = Some(item_ui_handle.clone());
        resync_global_chat_events.send(ResyncGlobalChatEvent::new(true));
    }

    pub fn recv_message(
        &mut self,
        resync_global_chat_events: &mut EventWriter<ResyncGlobalChatEvent>,
        message_id: MessageId,
        message_entity: Entity,
    ) {
        self.global_chats.insert(message_id, message_entity);

        if self.global_chats.len() > 100 {
            self.global_chats.pop_first();
        }

        resync_global_chat_events.send(ResyncGlobalChatEvent::new(true));
    }

    pub fn sync_with_collection(
        &mut self,
        session_client: &SessionClient,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        user_q: &Query<&UserPublic>,
        message_q: &Query<&MessagePublic>,
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
                let message_user_entity = message.owner_user_entity.get(session_client).unwrap();

                let (prev_timestamp_opt, prev_message_user_entity) = match prev_message_id_opt {
                    Some(prev_message_id) => {
                        let prev_message_entity =
                            *(self.global_chats.get(&prev_message_id).unwrap());
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
        message: &MessagePublic,
    ) {
        item_ctx.add_copied_node(ui);

        let divider_date_str = message.timestamp.date_string();
        item_ctx.set_text_by_id("timestamp", divider_date_str.as_str());
    }

    fn add_username_and_message_item(
        session_client: &SessionClient,
        user_q: &Query<&UserPublic>,
        item_ctx: &mut ListUiExtItem<MessageId>,
        ui: &UiHandle,
        message: &MessagePublic,
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
