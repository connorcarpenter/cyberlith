pub mod events;

mod ui_catalog;
pub use ui_catalog::UiCatalog;

mod main_menu;

mod plugin;
mod join_match;

pub use plugin::UiPlugin;

use bevy_ecs::{
    event::EventWriter,
    prelude::NextState,
};

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
    ui::{join_match::reset_join_match_state, events::{
        ResyncMessageListUiEvent, ResyncLobbyListUiEvent,
        ResyncUserListUiEvent,
    }},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UiKey {
    MainMenu,
    UserListItem,

    HostMatch,

    JoinMatch,
    JoinMatchLobbyItem,

    GlobalChat,
    GlobalChatDayDivider,
    GlobalChatUsernameAndMessage,
    GlobalChatMessage,

    Devlog,

    Settings,
}

pub(crate) fn on_ui_load(
    state: AppState,
    next_state: &mut NextState<AppState>,
    ui_manager: &mut UiManager,
    ui_catalog: &mut UiCatalog,
    user_manager: &mut UserManager,
    chat_message_manager: &mut ChatMessageManager,
    lobby_manager: &mut LobbyManager,
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
        UiKey::MainMenu => main_menu::on_load(state, next_state, ui_catalog, ui_manager, user_manager),
        UiKey::UserListItem => user_manager.on_load_user_list_item_ui(ui_catalog, resync_user_list_ui_events),

        UiKey::HostMatch => lobby_manager.on_load_host_match_ui(ui_catalog, ui_manager),

        UiKey::JoinMatch => lobby_manager.on_load_lobby_list_ui(ui_catalog, ui_manager, resync_lobby_list_ui_events),
        UiKey::JoinMatchLobbyItem => lobby_manager.on_load_lobby_item_ui(ui_catalog, resync_lobby_list_ui_events),

        UiKey::GlobalChat => chat_message_manager.on_load_container_ui(
            ui_catalog,
            ui_manager,
            resync_message_list_ui_events,
        ),
        UiKey::GlobalChatDayDivider => chat_message_manager.on_load_day_divider_item_ui(ui_catalog, resync_message_list_ui_events),
        UiKey::GlobalChatUsernameAndMessage => chat_message_manager
            .on_load_username_and_message_item_ui(ui_catalog, resync_message_list_ui_events),
        UiKey::GlobalChatMessage => chat_message_manager.on_load_message_item_ui(ui_catalog, resync_message_list_ui_events),

        _ => {
            unimplemented!("ui not implemented");
        }
    }
}

pub(crate) fn go_to_sub_ui(ui_manager: &mut UiManager, ui_catalog: &UiCatalog, sub_ui_key: UiKey) {
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
            UiKey::HostMatch => {
                LobbyManager::reset_host_match_state(ui_manager, &current_ui_handle)
            }
            UiKey::JoinMatch => {
                reset_join_match_state(ui_manager, &current_ui_handle)
            }
            UiKey::GlobalChat => ChatMessageManager::reset_state(ui_manager, &current_ui_handle),
            _ => {
                unimplemented!("ui not implemented");
            }
        }
    }

    ui_manager.set_ui_container_contents(&active_ui_handle, "center_container", &sub_ui_handle);
}
