pub mod events;

mod ui_catalog;
pub use ui_catalog::UiCatalog;

pub(crate) mod main_menu;
pub(crate) mod host_match;
pub(crate) mod global_chat;

use std::time::Duration;

use bevy_ecs::{
    event::EventReader,
    system::{Res, ResMut},
};

use game_engine::{
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    ui::{UiHandle, UiManager},
};

use crate::ui::events::{DevlogButtonClickedEvent, GlobalChatButtonClickedEvent, HostMatchButtonClickedEvent, JoinMatchButtonClickedEvent, SettingsButtonClickedEvent, SubmitButtonClickedEvent};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UiKey {
    MainMenu,
    HostMatch,
    JoinMatch,
    GlobalChat,
    Devlog,
    Settings,
}

pub(crate) fn handle_events(
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    ui_manager: Res<UiManager>,
    mut rumble_manager: ResMut<RumbleManager>,

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
    if ui_catalog.get_ui_key(&active_ui_handle) != &UiKey::MainMenu {
        panic!("unexpected ui");
    }

    let mut should_rumble = false;

    main_menu::handle_events(
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
            UiKey::HostMatch => {
                host_match::handle_events(&mut submit_btn_rdr, &mut should_rumble);
            }
            UiKey::GlobalChat => {
                global_chat::handle_events(&mut should_rumble);
            }
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
    ui_handle: &UiHandle
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != &UiKey::MainMenu {
        panic!("invalid sub-ui");
    }
    if let Some(current_ui_handle) = ui_manager.get_ui_container_contents(&active_ui_handle, "center_container") {
        match ui_catalog.get_ui_key(&current_ui_handle) {
            UiKey::MainMenu => panic!("invalid sub-ui"),
            UiKey::HostMatch => {
                host_match::reset_state(ui_manager, &current_ui_handle);
            }
            _ => {
                unimplemented!("ui not implemented");
            }
        }
    }

    ui_manager.set_ui_container_contents(&active_ui_handle, "center_container", ui_handle);
}
