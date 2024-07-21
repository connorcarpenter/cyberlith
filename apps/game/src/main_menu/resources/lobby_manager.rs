use std::collections::{BTreeMap, HashMap};

use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    system::{Query, Resource},
};

use game_engine::{
    asset::AssetManager,
    logging::info,
    session::{
        channels,
        components::{Lobby, User},
        messages, SessionClient,
    },
    social::LobbyId,
    ui::{
        extensions::{ListUiExt, ListUiExtItem},
        UiHandle, UiManager,
    },
};

use crate::main_menu::{ui::{
    events::{
        GoToSubUiEvent, LeaveLobbyButtonClickedEvent, LobbyListItemClickedEvent, ResyncLobbyListUiEvent,
        ResyncMainMenuUiEvent, ResyncMessageListUiEvent, ResyncUserListUiEvent, StartMatchButtonClickedEvent, SubmitButtonClickedEvent,
    },
    go_to_sub_ui, UiCatalog, UiKey,
}, resources::match_manager::MatchManager};

#[derive(Resource)]
pub struct LobbyManager {
    current_lobby: Option<LobbyId>,
    lobby_entities: BTreeMap<LobbyId, Entity>,
    list_ui_ext: ListUiExt<LobbyId>,
    lobby_item_ui: Option<UiHandle>,

    // user entity -> lobby id
    user_lobby_membership_map: HashMap<Entity, LobbyId>,

    // lobby_member_entity -> user_entity
    lobby_member_to_user_map: HashMap<Entity, Entity>
}

impl Default for LobbyManager {
    fn default() -> Self {
        Self {
            current_lobby: None,
            lobby_entities: BTreeMap::new(),
            list_ui_ext: ListUiExt::new(true),
            lobby_item_ui: None,
            user_lobby_membership_map: HashMap::new(),
            lobby_member_to_user_map: HashMap::new(),
        }
    }
}

impl LobbyManager {
    pub(crate) fn get_current_lobby(&self) -> Option<LobbyId> {
        self.current_lobby
    }

    pub(crate) fn set_current_lobby(
        &mut self,
        resync_main_menu_ui_events: &mut EventWriter<ResyncMainMenuUiEvent>,
        resync_chat_message_ui_events: &mut EventWriter<ResyncMessageListUiEvent>,
        resync_user_ui_events: &mut EventWriter<ResyncUserListUiEvent>,
        lobby_id: LobbyId,
    ) {
        if self.current_lobby.is_some() {
            panic!("current_lobby is already set!");
        }
        self.current_lobby = Some(lobby_id);

        resync_main_menu_ui_events.send(ResyncMainMenuUiEvent);
        resync_chat_message_ui_events.send(ResyncMessageListUiEvent::new(false));
        resync_user_ui_events.send(ResyncUserListUiEvent);
    }

    pub(crate) fn leave_current_lobby(
        &mut self,
        resync_main_menu_ui_events: &mut EventWriter<ResyncMainMenuUiEvent>,
        resync_chat_message_ui_events: &mut EventWriter<ResyncMessageListUiEvent>,
        resync_user_ui_events: &mut EventWriter<ResyncUserListUiEvent>,
    ) {
        if self.current_lobby.is_none() {
            panic!("current_lobby is not set!");
        }
        self.current_lobby = None;

        resync_main_menu_ui_events.send(ResyncMainMenuUiEvent);
        resync_chat_message_ui_events.send(ResyncMessageListUiEvent::new(false));
        resync_user_ui_events.send(ResyncUserListUiEvent);
    }

    pub(crate) fn get_lobby_entity(&self, lobby_id: &LobbyId) -> Option<Entity> {
        self.lobby_entities.get(lobby_id).copied()
    }

    pub(crate) fn put_user_in_lobby(&mut self, user_entity: Entity, lobby_id: LobbyId, lobby_member_entity: Entity) {
        if self.user_lobby_membership_map.contains_key(&user_entity) {
            panic!("user is already in a lobby!");
        }
        self.user_lobby_membership_map.insert(user_entity, lobby_id);

        if self.lobby_member_to_user_map.contains_key(&lobby_member_entity) {
            panic!("lobby member is already registered!");
        }
        self.lobby_member_to_user_map.insert(lobby_member_entity, user_entity);
    }

    // returns (lobby_id, user_entity)
    pub(crate) fn remove_user_from_lobby(&mut self, lobby_member_entity: &Entity) -> (LobbyId, Entity) {

        let Some(user_entity) = self.lobby_member_to_user_map.remove(lobby_member_entity) else {
            panic!("lobby member is not registered!");
        };
        let Some(lobby_id) = self.user_lobby_membership_map.remove(&user_entity) else {
            panic!("user is not in a lobby!");
        };
        (lobby_id, user_entity)
    }

    pub(crate) fn user_is_in_lobby(&self, user_entity: &Entity, lobby_entity: &Entity) -> bool {
        match self.user_lobby_membership_map.get(user_entity) {
            Some(user_lobby_id) => {
                let Some(user_lobby_entity) = self.lobby_entities.get(user_lobby_id) else {
                    return false;
                };
                user_lobby_entity == lobby_entity
            },
            None => false,
        }
    }

    pub(crate) fn handle_host_match_events(
        &mut self,
        ui_manager: &mut UiManager,
        ui_catalog: &UiCatalog,
        session_client: &mut SessionClient,
        sub_ui_event_writer: &mut EventWriter<GoToSubUiEvent>,
        submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
        should_rumble: &mut bool,
    ) {
        // Submit Button Click
        let mut submit_clicked = false;
        for _ in submit_btn_rdr.read() {
            submit_clicked = true;
        }
        if submit_clicked {
            info!("submit button clicked!");

            let ui_handle = ui_catalog.get_ui_handle(UiKey::HostMatch);

            // get name textbox text
            let Some(textbox_text) = ui_manager.get_text(&ui_handle, "name_textbox") else {
                return;
            };

            // clear name textbox
            ui_manager.set_text(&ui_handle, "name_textbox", "");

            // info!("Creating Match Lobby: {:?}", textbox_text);

            // send request to session server
            let message = messages::MatchLobbyCreate::new(&textbox_text);
            session_client.send_message::<channels::ClientActionsChannel, _>(&message);

            go_to_sub_ui(sub_ui_event_writer, UiKey::MessageList);

            // def rumble
            *should_rumble = true;
        }
    }

    pub(crate) fn handle_start_match_events(
        &mut self,
        session_client: &mut SessionClient,
        match_manager: &mut MatchManager,
        resync_main_menu_ui_events: &mut EventWriter<ResyncMainMenuUiEvent>,
        start_match_btn_rdr: &mut EventReader<StartMatchButtonClickedEvent>,
        should_rumble: &mut bool,
    ) {
        if match_manager.in_match() {
            return;
        }
        // Start Match Button Click
        let mut start_match_clicked = false;
        for _ in start_match_btn_rdr.read() {
            start_match_clicked = true;
        }
        if start_match_clicked {
            info!("start match button clicked!");

            // send request to session server
            session_client.send_message::<channels::ClientActionsChannel, _>(&messages::MatchLobbyGameStart);

            match_manager.start_match(resync_main_menu_ui_events);

            // def rumble
            *should_rumble = true;
        }
    }

    pub(crate) fn handle_leave_lobby_events(
        &mut self,
        session_client: &mut SessionClient,
        resync_main_menu_ui_events: &mut EventWriter<ResyncMainMenuUiEvent>,
        resync_chat_message_ui_events: &mut EventWriter<ResyncMessageListUiEvent>,
        resync_user_ui_events: &mut EventWriter<ResyncUserListUiEvent>,
        leave_lobby_btn_rdr: &mut EventReader<LeaveLobbyButtonClickedEvent>,
        should_rumble: &mut bool,
    ) {
        if self.current_lobby.is_none() {
            return;
        }
        // Leavy Lobby Button Click
        let mut leave_lobby_clicked = false;
        for _ in leave_lobby_btn_rdr.read() {
            leave_lobby_clicked = true;
        }
        if leave_lobby_clicked {
            info!("leave lobby button clicked!");

            // send request to session server
            session_client.send_message::<channels::ClientActionsChannel, _>(&messages::MatchLobbyLeave);

            // set current_lobby prematurely to de-dupe requests
            self.leave_current_lobby(resync_main_menu_ui_events, resync_chat_message_ui_events, resync_user_ui_events);

            // def rumble
            *should_rumble = true;
        }
    }

    pub(crate) fn on_load_lobby_list_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        resync_match_lobbies_events: &mut EventWriter<ResyncLobbyListUiEvent>,
    ) {
        let ui_key = UiKey::JoinMatch;
        let ui_handle = ui_catalog.get_ui_handle(ui_key);

        ui_catalog.set_loaded(ui_key);

        // setup lobby list extension
        {
            let container_id_str = "lobby_list";

            self.list_ui_ext
                .set_container_ui(ui_manager, &ui_handle, container_id_str);
            resync_match_lobbies_events.send(ResyncLobbyListUiEvent);
        }
    }

    pub(crate) fn on_load_lobby_item_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        resync_match_lobbies_events: &mut EventWriter<ResyncLobbyListUiEvent>,
    ) {
        let item_ui_key = UiKey::JoinMatchLobbyItem;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        self.lobby_item_ui = Some(item_ui_handle.clone());
        resync_match_lobbies_events.send(ResyncLobbyListUiEvent);
    }

    pub fn recv_lobby(
        &mut self,
        lobby_id: LobbyId,
        lobby_entity: Entity,
        resync_lobby_ui_events: &mut EventWriter<ResyncLobbyListUiEvent>,
    ) {
        self.lobby_entities.insert(lobby_id, lobby_entity);
        // info!("LobbyManager recv_lobby(): {:?}", self.lobby_entities);

        resync_lobby_ui_events.send(ResyncLobbyListUiEvent);
    }

    pub fn remove_lobby(
        &mut self,
        lobby_id: LobbyId,
        resync_lobby_ui_events: &mut EventWriter<ResyncLobbyListUiEvent>,
    ) {
        self.lobby_entities.remove(&lobby_id);
        // info!("LobbyManager remove_lobby(): {:?}", self.lobby_entities);

        resync_lobby_ui_events.send(ResyncLobbyListUiEvent);
    }

    pub fn scroll_up(&mut self) {
        self.list_ui_ext.scroll_up();
    }

    pub fn scroll_down(&mut self) {
        self.list_ui_ext.scroll_up();
    }

    pub fn sync_with_collection(
        &mut self,
        session_client: &SessionClient,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        user_q: &Query<&User>,
        lobby_q: &Query<&Lobby>,
    ) {
        if self.lobby_item_ui.is_none() {
            return;
        }

        let lobby_ui_handle = self.lobby_item_ui.as_ref().unwrap();

        // info!("LobbyManager sync_with_collection(): {:?}", self.lobby_entities);
        self.list_ui_ext.sync_with_collection(
            ui_manager,
            asset_manager,
            self.lobby_entities.iter(),
            self.lobby_entities.len(),
            |item_ctx, lobby_id, _| {

                // info!("LobbyManager sync_with_collection() lobby_id: {:?}", lobby_id);

                let lobby_entity = *(self.lobby_entities.get(&lobby_id).unwrap());
                let lobby = lobby_q.get(lobby_entity).unwrap();

                // filter out lobbies that are in progress
                if lobby.is_in_progress() {
                    return;
                }

                let lobby_owner_entity = lobby.owner_user_entity.get(session_client).unwrap();
                let owner_user = user_q.get(lobby_owner_entity).unwrap();

                Self::add_lobby_item(
                    item_ctx,
                    lobby_ui_handle,
                    lobby.name.as_str(),
                    owner_user.name.as_str(),
                );
            },
        );
    }

    fn add_lobby_item(
        item_ctx: &mut ListUiExtItem<LobbyId>,
        ui: &UiHandle,
        lobby_name: &str,
        owner_name: &str,
    ) {
        item_ctx.add_copied_node(ui);

        item_ctx.set_text_by_id("match_name", lobby_name);
        item_ctx.set_text_by_id("username", owner_name);
        item_ctx.register_ui_event::<LobbyListItemClickedEvent>("lobby_button"); // uses container_ui_handle
    }
}
