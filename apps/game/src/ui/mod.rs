pub mod events;

mod ui_catalog;
pub use ui_catalog::UiCatalog;

mod main_menu;

mod host_match;
mod join_match;
mod message_list;
mod plugin;
mod user_list;

pub use plugin::UiPlugin;

use bevy_ecs::{system::{Res, ResMut}, event::{EventReader, EventWriter}, prelude::NextState};

use game_engine::{
    asset::AssetId,
    ui::{UiHandle, UiManager},
};

use crate::{
    resources::{
        chat_message_manager::ChatMessageManager, lobby_manager::LobbyManager,
        user_manager::UserManager,
    },
    states::AppState,
    ui::events::{GoToSubUiEvent, ResyncLobbyListUiEvent, ResyncMessageListUiEvent, ResyncUserListUiEvent},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UiKey {
    MainMenu,
    UserListItem,

    HostMatch,

    JoinMatch,
    JoinMatchLobbyItem,

    MessageList,
    MessageListDayDivider,
    MessageListUsernameAndMessage,
    MessageListMessage,

    Devlog,

    Settings,

    Invalid,
}

impl Default for UiKey {
    fn default() -> Self {
        UiKey::Invalid
    }
}

pub(crate) fn on_ui_load(
    state: AppState,
    next_state: &mut NextState<AppState>,
    ui_manager: &mut UiManager,
    ui_catalog: &mut UiCatalog,
    user_manager: &mut UserManager,
    chat_message_manager: &mut ChatMessageManager,
    lobby_manager: &mut LobbyManager,
    sub_ui_event_writer: &mut EventWriter<GoToSubUiEvent>,
    resync_user_list_ui_events: &mut EventWriter<ResyncUserListUiEvent>,
    resync_message_list_ui_events: &mut EventWriter<ResyncMessageListUiEvent>,
    resync_lobby_list_ui_events: &mut EventWriter<ResyncLobbyListUiEvent>,
    asset_id: AssetId,
) {
    let ui_handle = UiHandle::new(asset_id);
    if !ui_catalog.has_ui_key(&ui_handle) {
        panic!("ui is not registered in catalog");
    }
    let ui_key = ui_catalog.get_ui_key(&ui_handle);

    match ui_key {
        UiKey::MainMenu => main_menu::on_main_menu_ui_load(state, next_state, ui_catalog, ui_manager, user_manager, sub_ui_event_writer),
        UiKey::UserListItem => user_manager.on_load_user_list_item_ui(ui_catalog, resync_user_list_ui_events),

        UiKey::HostMatch => host_match::on_load_host_match_ui(ui_catalog, ui_manager),

        UiKey::JoinMatch => lobby_manager.on_load_lobby_list_ui(ui_catalog, ui_manager, resync_lobby_list_ui_events),
        UiKey::JoinMatchLobbyItem => lobby_manager.on_load_lobby_item_ui(ui_catalog, resync_lobby_list_ui_events),

        UiKey::MessageList => chat_message_manager.on_load_container_ui(
            ui_catalog,
            ui_manager,
            resync_message_list_ui_events,
            sub_ui_event_writer,
        ),
        UiKey::MessageListDayDivider => chat_message_manager
            .on_load_day_divider_item_ui(ui_catalog, resync_message_list_ui_events),
        UiKey::MessageListUsernameAndMessage => chat_message_manager
            .on_load_username_and_message_item_ui(ui_catalog, resync_message_list_ui_events),
        UiKey::MessageListMessage => chat_message_manager.on_load_message_item_ui(ui_catalog, resync_message_list_ui_events),

        _ => {
            unimplemented!("ui not implemented");
        }
    }
}

pub(crate) fn go_to_sub_ui(
    sub_ui_event_writer: &mut EventWriter<GoToSubUiEvent>,
    sub_ui_key: UiKey,
) {
    sub_ui_event_writer.send(GoToSubUiEvent(sub_ui_key));
}

pub(crate) fn process_go_to_sub_ui_events(
    mut ui_manager: ResMut<UiManager>,
    ui_catalog: Res<UiCatalog>,
    mut resync_lobby_list_ui_event_writer: EventWriter<ResyncLobbyListUiEvent>,
    mut resync_message_list_ui_event_writer: EventWriter<ResyncMessageListUiEvent>,
    mut sub_ui_event_reader: EventReader<GoToSubUiEvent>,
) {
    let mut sub_ui_key = None;

    for sub_ui_event in sub_ui_event_reader.read() {
        sub_ui_key = Some(sub_ui_event.0);
    }

    if sub_ui_key.is_none() {
        return;
    }
    let sub_ui_key = sub_ui_key.unwrap();

    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("invalid sub-ui");
    }
    let sub_ui_handle = ui_catalog.get_ui_handle(sub_ui_key);
    if !ui_catalog.get_is_loaded(sub_ui_key) {
        panic!("ui not loaded");
    }
    if let Some(current_ui_handle) =
        ui_manager.get_ui_container_contents(&active_ui_handle, "center_container")
    {
        match ui_catalog.get_ui_key(&current_ui_handle) {
            UiKey::MainMenu => panic!("invalid sub-ui"),
            UiKey::HostMatch => host_match::on_leave_state(&mut ui_manager, &current_ui_handle),
            UiKey::JoinMatch => join_match::on_leave_state(&mut ui_manager, &current_ui_handle),
            UiKey::MessageList => message_list::on_leave_state(&mut ui_manager, &current_ui_handle),
            _ => {
                unimplemented!("ui not implemented");
            }
        }
    }

    ui_manager.set_ui_container_contents(&active_ui_handle, "center_container", &sub_ui_handle);

    match sub_ui_key {
        UiKey::MainMenu => panic!("invalid sub-ui"),
        UiKey::HostMatch => host_match::on_enter_state(&mut ui_manager, &sub_ui_handle),
        UiKey::JoinMatch => join_match::on_enter_state(&mut resync_lobby_list_ui_event_writer),
        UiKey::MessageList => message_list::on_enter_state(&mut resync_message_list_ui_event_writer),
        _ => {
            unimplemented!("ui not implemented");
        }
    }
}
