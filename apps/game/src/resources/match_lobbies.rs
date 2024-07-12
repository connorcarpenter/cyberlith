use std::collections::BTreeMap;

use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    system::{Query, Resource},
};

use game_engine::{
    asset::AssetManager,
    input::{InputEvent, Key},
    logging::{info},
    session::{channels, components::{MatchLobby, PublicUserInfo}, messages, SessionClient},
    social::{MatchLobbyId},
    ui::{
        extensions::{ListUiExt, ListUiExtItem},
        UiHandle, UiManager,
    },
};

use crate::ui::{UiCatalog, UiKey, events::{ResyncMatchLobbiesEvent, SubmitButtonClickedEvent}};

#[derive(Resource)]
pub struct MatchLobbies {
    match_lobbies: BTreeMap<MatchLobbyId, Entity>,
    list_ui_ext: ListUiExt<MatchLobbyId>,
    lobby_item_ui: Option<UiHandle>,
}

impl Default for MatchLobbies {
    fn default() -> Self {
        Self {
            match_lobbies: BTreeMap::new(),
            list_ui_ext: ListUiExt::new(true),
            lobby_item_ui: None,
        }
    }
}

impl MatchLobbies {

    pub(crate) fn handle_host_match_events(
        &mut self,
        ui_manager: &mut UiManager,
        ui_catalog: &UiCatalog,
        session_server: &mut SessionClient,
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
            let Some(textbox_text) = ui_manager.get_textbox_text(&ui_handle, "name_textbox") else {
                return;
            };

            // clear name textbox
            ui_manager.set_textbox_text(&ui_handle, "name_textbox", "");

            // info!("Creating Match Lobby: {:?}", textbox_text);

            // send request to session server
            let message = messages::MatchLobbyCreate::new(&textbox_text);
            session_server.send_message::<channels::ClientActionsChannel, _>(&message);

            // TODO: GO TO MATCH LOBBY INSIDE!

            // def rumble
            *should_rumble = true;
        }
    }

    pub(crate) fn handle_join_match_events(
        &mut self,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        session_server: &mut SessionClient,
        user_q: &Query<&PublicUserInfo>,
        lobby_q: &Query<&MatchLobby>,
        input_events: &mut EventReader<InputEvent>,
        resync_match_lobbies_events: &mut EventReader<ResyncMatchLobbiesEvent>,
        _should_rumble: &mut bool,
    ) {
        let mut should_resync = false;
        for _resync_event in resync_match_lobbies_events.read() {
            should_resync = true;
        }

        for event in input_events.read() {
            match event {
                // TODO this probably doesn't belong here! this is where it is required to be selecting the textbox!!!
                InputEvent::KeyPressed(Key::I, _) => {
                    info!("I Key Pressed");

                    info!("Scrolling Up");
                    self.list_ui_ext.scroll_up();
                    should_resync = true;
                }
                InputEvent::KeyPressed(Key::J, _) => {
                    info!("J Key Pressed");

                    info!("Scrolling Down");
                    self.list_ui_ext.scroll_down();
                    should_resync = true;
                }
                InputEvent::KeyPressed(Key::Enter, _modifiers) => {
                    // TODO: enter into lobby?
                }
                _ => {}
            }
        }

        if should_resync {
            self.sync_with_collection(session_server, ui_manager, asset_manager, user_q, lobby_q);
        }
    }

    pub fn reset_host_match_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
        // TODO: implement
    }

    pub fn reset_join_match_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
        // TODO: implement
    }

    pub fn on_load_host_match_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager
    ) {
        let ui_key = UiKey::HostMatch;
        let ui_handle = ui_catalog.get_ui_handle(ui_key);

        ui_catalog.set_loaded(ui_key);
        ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&ui_handle, "submit_button");
    }

    pub(crate) fn on_load_lobby_list_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        ui_manager: &mut UiManager,
        resync_match_lobbies_events: &mut EventWriter<ResyncMatchLobbiesEvent>,
    ) {
        let ui_key = UiKey::JoinMatch;
        let ui_handle = ui_catalog.get_ui_handle(ui_key);

        ui_catalog.set_loaded(ui_key);

        // setup lobby list extension
        {
            let container_id_str = "lobby_list";

            self.list_ui_ext
                .set_container_ui(ui_manager, &ui_handle, container_id_str);
            resync_match_lobbies_events.send(ResyncMatchLobbiesEvent);
        }
    }

    pub(crate) fn on_load_lobby_item_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        resync_match_lobbies_events: &mut EventWriter<ResyncMatchLobbiesEvent>,
    ) {
        let item_ui_key = UiKey::JoinMatchLobbyItem;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        self.lobby_item_ui = Some(item_ui_handle.clone());
        resync_match_lobbies_events.send(ResyncMatchLobbiesEvent);
    }

    pub fn recv_lobby(
        &mut self,
        resync_match_lobbies_events: &mut EventWriter<ResyncMatchLobbiesEvent>,
        lobby_id: MatchLobbyId,
        lobby_entity: Entity,
    ) {
        self.match_lobbies.insert(lobby_id, lobby_entity);

        resync_match_lobbies_events.send(ResyncMatchLobbiesEvent);
    }

    pub fn remove_lobby(
        &mut self,
        resync_match_lobbies_events: &mut EventWriter<ResyncMatchLobbiesEvent>,
        lobby_id: MatchLobbyId,
    ) {
        self.match_lobbies.remove(&lobby_id);

        resync_match_lobbies_events.send(ResyncMatchLobbiesEvent);
    }

    pub fn sync_with_collection(
        &mut self,
        session_client: &SessionClient,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        user_q: &Query<&PublicUserInfo>,
        lobby_q: &Query<&MatchLobby>,
    ) {
        if self.lobby_item_ui.is_none() {
            return;
        }

        let lobby_ui_handle = self.lobby_item_ui.as_ref().unwrap();

        self.list_ui_ext.sync_with_collection(
            ui_manager,
            asset_manager,
            self.match_lobbies.iter(),
            self.match_lobbies.len(),
            |item_ctx, lobby_id, _| {
                let lobby_entity = *(self.match_lobbies.get(&lobby_id).unwrap());
                let lobby = lobby_q.get(lobby_entity).unwrap();

                let lobby_owner_entity = lobby.user_entity.get(session_client).unwrap();
                let owner_info = user_q.get(lobby_owner_entity).unwrap();

                Self::add_lobby_item(item_ctx, lobby_ui_handle, lobby.name.as_str(), owner_info.name.as_str());
            },
        );
    }

    fn add_lobby_item(
        item_ctx: &mut ListUiExtItem<MatchLobbyId>,
        ui: &UiHandle,
        lobby_name: &str,
        owner_name: &str,
    ) {
        item_ctx.add_copied_node(ui);

        item_ctx.set_text_by_id("match_name", lobby_name);
        item_ctx.set_text_by_id("username", owner_name);
    }
}
