pub mod events;

mod ui_catalog;
pub use ui_catalog::UiCatalog;

mod main_menu;
mod host_match;

use std::time::Duration;

use bevy_ecs::{
    event::EventReader,
    system::{Res, ResMut},
    prelude::NextState
};
use bevy_ecs::system::Query;

use game_engine::{
    input::{InputEvent, GamepadRumbleIntensity, Input, RumbleManager},
    ui::{UiManager, UiHandle},
    asset::AssetId,
    session::SessionClient,
};
use game_engine::session::components::GlobalChatMessage;

use crate::{resources::global_chat::GlobalChat, states::AppState, ui::events::{DevlogButtonClickedEvent, GlobalChatButtonClickedEvent, HostMatchButtonClickedEvent, JoinMatchButtonClickedEvent, SettingsButtonClickedEvent, SubmitButtonClickedEvent}};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UiKey {
    MainMenu,
    HostMatch,
    JoinMatch,
    GlobalChat,
    GlobalChatList,
    GlobalChatListItem,
    Devlog,
    Settings,
}

pub(crate) fn on_ui_load(
    state: AppState,
    next_state: &mut NextState<AppState>,
    ui_manager: &mut UiManager,
    ui_catalog: &mut UiCatalog,
    global_chat_messages: &mut GlobalChat,
    message_q: &Query<&GlobalChatMessage>,
    asset_id: AssetId
) {
    let ui_handle = UiHandle::new(asset_id);
    if !ui_catalog.has_ui_key(&ui_handle) {
        panic!("ui is not registered in catalog");
    }
    let ui_key = ui_catalog.get_ui_key(&ui_handle);

    match ui_key {
        UiKey::MainMenu => main_menu::on_load(
            state,
            next_state,
            ui_catalog,
            ui_manager,
        ),
        UiKey::HostMatch => host_match::on_load(
            ui_catalog,
            ui_manager,
        ),
        UiKey::GlobalChat => GlobalChat::on_load_container_ui(
            ui_catalog,
            ui_manager,
            &message_q,
            global_chat_messages,
        ),
        UiKey::GlobalChatList=> GlobalChat::on_load_list_ui(
            ui_catalog,
            ui_manager,
            &message_q,
            global_chat_messages,
        ),
        UiKey::GlobalChatListItem => GlobalChat::on_load_list_item_ui(
            ui_catalog,
            ui_manager,
            &message_q,
            global_chat_messages,
        ),
        _ => {
            unimplemented!("ui not implemented");
        }
    }
}

pub(crate) fn handle_events(
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    mut ui_manager: ResMut<UiManager>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut session_client: SessionClient,

    mut input_events: EventReader<InputEvent>,
    mut host_match_btn_rdr: EventReader<HostMatchButtonClickedEvent>,
    mut join_match_btn_rdr: EventReader<JoinMatchButtonClickedEvent>,
    mut global_chat_btn_rdr: EventReader<GlobalChatButtonClickedEvent>,
    mut devlog_btn_rdr: EventReader<DevlogButtonClickedEvent>,
    mut settings_btn_rdr: EventReader<SettingsButtonClickedEvent>,
    mut submit_btn_rdr: EventReader<SubmitButtonClickedEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("unexpected ui");
    }

    let mut should_rumble = false;

    main_menu::handle_events(
        &mut ui_manager,
        &ui_catalog,
        &mut host_match_btn_rdr,
        &mut join_match_btn_rdr,
        &mut global_chat_btn_rdr,
        &mut devlog_btn_rdr,
        &mut settings_btn_rdr,
        &mut should_rumble,
    );

    if let Some(current_ui_handle) = ui_manager.get_ui_container_contents(&active_ui_handle, "center_container") {
        match ui_catalog.get_ui_key(&current_ui_handle) {
            UiKey::MainMenu => panic!("invalid sub-ui"),
            UiKey::HostMatch => host_match::handle_events(
                &mut submit_btn_rdr,
                &mut should_rumble
            ),
            UiKey::GlobalChat => GlobalChat::handle_events(
                &mut ui_manager,
                &ui_catalog,
                &mut session_client,
                &mut input_events,
                &mut should_rumble,
            ),
            _ => {
                unimplemented!("ui not implemented");
            }
        }
    };

    // handle rumble
    if should_rumble {
        if let Some(id) = input.gamepad_first() {
            rumble_manager.add_rumble(
                id,
                Duration::from_millis(200),
                GamepadRumbleIntensity::strong_motor(0.4),
            );
        }
    }

    // drain all events
    for _ in host_match_btn_rdr.read() {}
    for _ in join_match_btn_rdr.read() {}
    for _ in global_chat_btn_rdr.read() {}
    for _ in devlog_btn_rdr.read() {}
    for _ in settings_btn_rdr.read() {}
}

pub(crate) fn go_to_sub_ui(
    ui_manager: &mut UiManager,
    ui_catalog: &UiCatalog,
    sub_ui_key: UiKey,
) {
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
    if let Some(current_ui_handle) = ui_manager.get_ui_container_contents(&active_ui_handle, "center_container") {
        match ui_catalog.get_ui_key(&current_ui_handle) {
            UiKey::MainMenu => panic!("invalid sub-ui"),
            UiKey::HostMatch => host_match::reset_state(ui_manager, &current_ui_handle),
            UiKey::GlobalChat => GlobalChat::reset_state(ui_manager, &current_ui_handle),
            _ => {
                unimplemented!("ui not implemented");
            }
        }
    }

    ui_manager.set_ui_container_contents(&active_ui_handle, "center_container", &sub_ui_handle);
}
