use std::collections::BTreeMap;

use game_engine::{
    asset::AssetManager,
    ui::{
        extensions::{ListUiExt, ListUiExtItem},
        UiHandle, UiManager,
    },
};

use bevy_ecs::system::Resource;

use crate::app::{global::Global, uis::game};

#[derive(Resource)]
pub struct MatchLobbyListState {
    pub match_lobbies_list_ui_ext: ListUiExt<u32>,

    // username, lobby name
    pub match_lobbies: BTreeMap<u32, (String, String, bool)>,

    match_lobby_list_item_ui_handle: UiHandle,
}

impl MatchLobbyListState {
    pub fn new(item_ui_handle: &UiHandle) -> Self {
        Self {
            match_lobbies_list_ui_ext: ListUiExt::new(false),
            match_lobbies: BTreeMap::new(),

            match_lobby_list_item_ui_handle: item_ui_handle.clone(),
        }
    }

    pub fn match_lobbies_scroll_up(
        &mut self,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
    ) {
        self.match_lobbies_list_ui_ext.scroll_up();

        self.sync_lobbies_collections(ui_manager, asset_manager);
    }

    pub fn match_lobbies_scroll_down(
        &mut self,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
    ) {
        self.match_lobbies_list_ui_ext.scroll_down();

        self.sync_lobbies_collections(ui_manager, asset_manager);
    }

    pub fn sync_lobbies_collections(
        &mut self,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
    ) {
        // setup collection
        self.match_lobbies_list_ui_ext.sync_with_collection(
            ui_manager,
            asset_manager,
            self.match_lobbies.iter(),
            self.match_lobbies.len(),
            |item_ctx, lobby_id, _| {
                let (username, lobbyname, disabled) = self.match_lobbies.get(&lobby_id).unwrap();

                // just add message
                add_lobby_item(
                    item_ctx,
                    &self.match_lobby_list_item_ui_handle,
                    username,
                    lobbyname,
                    *disabled,
                );
            },
        );
    }
}

fn add_lobby_item(
    item_ctx: &mut ListUiExtItem<u32>,
    ui: &UiHandle,
    username: &str,
    lobby_name: &str,
    disabled: bool,
) {
    item_ctx.add_copied_node(ui);
    item_ctx.set_text_by_id("username", username);
    item_ctx.set_text_by_id("match_name", lobby_name);
    item_ctx.set_button_enabled("lobby_button", !disabled);
}

pub(crate) fn setup_match_lobby_test_case(
    global: &mut Global,
    ui_manager: &mut UiManager,
    asset_manager: &AssetManager,
    main_menu_ui_handle: &UiHandle,
) -> MatchLobbyListState {
    let join_match_ui_handle = global.load_ui(ui_manager, game::join_match::ui_define()); // game match lobby list
    let match_lobby_list_item_ui_handle =
        global.load_ui(ui_manager, game::match_lobby_list_item::ui_define()); // game match lobby list item

    let mut match_lobby_state = MatchLobbyListState::new(&match_lobby_list_item_ui_handle);

    // setup sub ui
    // ui_manager.set_ui_container_contents(
    //     &main_menu_ui_handle,
    //     "center_container",
    //     &join_match_ui_handle,
    // );

    // setup match lobby list
    match_lobby_state
        .match_lobbies_list_ui_ext
        .set_container_ui(ui_manager, &join_match_ui_handle, "lobby_list");

    // setup chats
    match_lobby_state.match_lobbies = setup_match_lobbies();

    match_lobby_state.sync_lobbies_collections(ui_manager, asset_manager);

    match_lobby_state
}

fn setup_match_lobbies() -> BTreeMap<u32, (String, String, bool)> {
    let mut users = Vec::new();
    users.push("tom");
    users.push("ben");
    users.push("andrew");
    users.push("joe");
    users.push("jane");
    users.push("sarah");
    users.push("jim");
    users.push("bob");

    let mut matchnames = Vec::new();
    matchnames.push("fun times only");
    matchnames.push("serious business");
    matchnames.push("noobs!!");
    matchnames.push("progamers baby");
    matchnames.push("snipin all day!!!");
    matchnames.push("no camping");

    let mut match_lobbies = BTreeMap::<u32, (String, String, bool)>::new();

    let mut current_user_index = 0;

    for _i in 0..64 {
        if random::gen_range_u32(0, 5) < 1 {
            current_user_index = random::gen_range_u32(0, users.len() as u32) as usize;
        }
        let matchname_index = random::gen_range_u32(0, matchnames.len() as u32) as usize;
        setup_global_chat(
            &mut match_lobbies,
            users[current_user_index],
            matchnames[matchname_index],
            random::gen_bool(),
        );
    }

    match_lobbies
}

fn setup_global_chat(
    match_lobbies: &mut BTreeMap<u32, (String, String, bool)>,
    username: &str,
    matchname: &str,
    disabled: bool,
) {
    let id = match_lobbies.len() as u32;
    let matchname_str = format!("{:?} {:?}", id, matchname);

    match_lobbies.insert(
        match_lobbies.len() as u32,
        (username.to_string(), matchname_str, disabled),
    );
}
