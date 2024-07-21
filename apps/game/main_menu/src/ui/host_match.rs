use std::time::Duration;

use bevy_ecs::{
    change_detection::{Res, ResMut},
    event::{EventReader, EventWriter},
};

use game_engine::{
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    session::SessionClient,
    ui::{UiHandle, UiManager},
};

use crate::{ui::{events::{GoToSubUiEvent, SubmitButtonClickedEvent}, UiCatalog, UiKey}, resources::lobby_manager::LobbyManager};

pub fn on_load_host_match_ui(ui_catalog: &mut UiCatalog, ui_manager: &mut UiManager) {
    let ui_key = UiKey::HostMatch;
    let ui_handle = ui_catalog.get_ui_handle(ui_key);

    ui_catalog.set_loaded(ui_key);
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&ui_handle, "submit_button");
}

pub(crate) fn handle_host_match_events(
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    mut ui_manager: ResMut<UiManager>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut session_client: SessionClient,
    mut lobby_manager: ResMut<LobbyManager>,
    mut sub_ui_event_writer: EventWriter<GoToSubUiEvent>,
    mut submit_btn_rdr: EventReader<SubmitButtonClickedEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        panic!("unexpected ui");
    }

    let mut should_rumble = false;

    if let Some(current_ui_handle) =
        ui_manager.get_ui_container_contents(&active_ui_handle, "center_container")
    {
        let ui_key = ui_catalog.get_ui_key(&current_ui_handle);
        if UiKey::HostMatch == ui_key {
            lobby_manager.handle_host_match_events(
                &mut ui_manager,
                &ui_catalog,
                &mut session_client,
                &mut sub_ui_event_writer,
                &mut submit_btn_rdr,
                &mut should_rumble,
            );
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
}

pub fn on_enter_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
    // TODO: implement
}

pub fn on_leave_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
    // TODO: implement
    // clear input fields
}
