use std::collections::BTreeMap;

use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    system::{Query, Resource},
};

use game_engine::{
    asset::AssetManager,
    input::{InputEvent, Key},
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

use crate::ui::{
    events::{ResyncLobbyUiEvent, SubmitButtonClickedEvent},
    UiCatalog, UiKey,
};

#[derive(Resource)]
pub struct LobbyManager {
    current_lobby: Option<LobbyId>,
    lobby_entities: BTreeMap<LobbyId, Entity>,
    list_ui_ext: ListUiExt<LobbyId>,
    lobby_item_ui: Option<UiHandle>,
}

impl Default for LobbyManager {
    fn default() -> Self {
        Self {
            current_lobby: None,
            lobby_entities: BTreeMap::new(),
            list_ui_ext: ListUiExt::new(true),
            lobby_item_ui: None,
        }
    }
}

impl LobbyManager {

    pub(crate) fn get_current_lobby_id(&self) -> Option<LobbyId> {
        self.current_lobby
    }

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
        user_q: &Query<&User>,
        lobby_q: &Query<&Lobby>,
        input_events: &mut EventReader<InputEvent>,
        resync_lobby_ui_events: &mut EventReader<ResyncLobbyUiEvent>,
        _should_rumble: &mut bool,
    ) {
        let mut should_resync = false;
        for _resync_event in resync_lobby_ui_events.read() {
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
        ui_manager: &mut UiManager,
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
        resync_match_lobbies_events: &mut EventWriter<ResyncLobbyUiEvent>,
    ) {
        let ui_key = UiKey::JoinMatch;
        let ui_handle = ui_catalog.get_ui_handle(ui_key);

        ui_catalog.set_loaded(ui_key);

        // setup lobby list extension
        {
            let container_id_str = "lobby_list";

            self.list_ui_ext
                .set_container_ui(ui_manager, &ui_handle, container_id_str);
            resync_match_lobbies_events.send(ResyncLobbyUiEvent);
        }
    }

    pub(crate) fn on_load_lobby_item_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        resync_match_lobbies_events: &mut EventWriter<ResyncLobbyUiEvent>,
    ) {
        let item_ui_key = UiKey::JoinMatchLobbyItem;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        self.lobby_item_ui = Some(item_ui_handle.clone());
        resync_match_lobbies_events.send(ResyncLobbyUiEvent);
    }

    pub fn recv_lobby(
        &mut self,
        lobby_id: LobbyId,
        lobby_entity: Entity,
        resync_lobby_ui_events: &mut EventWriter<ResyncLobbyUiEvent>,
    ) {
        self.lobby_entities.insert(lobby_id, lobby_entity);

        resync_lobby_ui_events.send(ResyncLobbyUiEvent);
    }

    pub fn remove_lobby(
        &mut self,
        lobby_id: LobbyId,
        resync_lobby_ui_events: &mut EventWriter<ResyncLobbyUiEvent>,
    ) {
        self.lobby_entities.remove(&lobby_id);

        resync_lobby_ui_events.send(ResyncLobbyUiEvent);
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

        self.list_ui_ext.sync_with_collection(
            ui_manager,
            asset_manager,
            self.lobby_entities.iter(),
            self.lobby_entities.len(),
            |item_ctx, lobby_id, _| {
                let lobby_entity = *(self.lobby_entities.get(&lobby_id).unwrap());
                let lobby = lobby_q.get(lobby_entity).unwrap();

                let lobby_owner_entity = lobby.owner_user_entity.get(session_client).unwrap();
                let owner_info = user_q.get(lobby_owner_entity).unwrap();

                Self::add_lobby_item(
                    item_ctx,
                    lobby_ui_handle,
                    lobby.name.as_str(),
                    owner_info.name.as_str(),
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
    }
}
