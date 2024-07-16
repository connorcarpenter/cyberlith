use std::time::Duration;

use bevy_ecs::{prelude::Query, event::EventReader, change_detection::{Res, ResMut}};

use game_engine::{ui::{UiManager, UiHandle}, session::{SessionClient, components::{ChatMessage, User}}, asset::AssetManager, input::{GamepadRumbleIntensity, Input, InputEvent, RumbleManager}};

use crate::{ui::{UiCatalog, UiKey, events::ResyncMessageListUiEvent}, resources::{chat_message_manager::ChatMessageManager}};

pub(crate) fn handle_resync_message_list_ui_events(
    ui_catalog: Res<UiCatalog>,
    input: Res<Input>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut session_client: SessionClient,
    mut message_manager: ResMut<ChatMessageManager>,
    user_q: Query<&User>,
    message_q: Query<&ChatMessage>,
    mut input_events: EventReader<InputEvent>,
    mut resync_global_chat_events: EventReader<ResyncMessageListUiEvent>,
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
        if UiKey::GlobalChat == ui_catalog.get_ui_key(&current_ui_handle) {
            message_manager.handle_events(
                &mut ui_manager,
                &ui_catalog,
                &asset_manager,
                &mut session_client,
                &mut input_events,
                &mut resync_global_chat_events,
                &user_q,
                &message_q,
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

pub fn reset_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
    // TODO: implement
}