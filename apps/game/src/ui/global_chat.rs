use bevy_ecs::event::EventReader;

use game_engine::{logging::info, ui::{UiHandle, UiManager, NodeActiveState}, input::{InputEvent, Key}, session::{SessionClient, messages, channels}};

use crate::ui::{go_to_sub_ui, UiCatalog, UiKey};

pub(crate) fn on_load(
    ui_catalog: &mut UiCatalog,
    ui_manager: &mut UiManager,
) {
    let ui_key = UiKey::GlobalChat;

    ui_catalog.set_loaded(ui_key);

    // set sub-ui to GlobalChat at beginning
    if let Some(active_ui_handle) = ui_manager.active_ui() {
        if ui_catalog.get_ui_key(&active_ui_handle) == UiKey::MainMenu {
            go_to_sub_ui(ui_manager, ui_catalog, UiKey::GlobalChat);
        }
    }
}

pub(crate) fn handle_events(
    ui_manager: &mut UiManager,
    ui_catalog: &UiCatalog,
    session_server: &mut SessionClient,
    input_events: &mut EventReader<InputEvent>,
    _should_rumble: &mut bool,
) {
    let ui_handle = ui_catalog.get_ui_handle(UiKey::GlobalChat);
    let Some(NodeActiveState::Active) = ui_manager.get_node_active_state_from_id(&ui_handle, "message_textbox") else {
        return;
    };

    for event in input_events.read() {
        match event {
            InputEvent::KeyPressed(Key::Enter, modifiers) => {
                if modifiers.shift {
                    // later, add multi-line newline
                } else {
                    // send message
                    send_message(ui_manager, &ui_handle, session_server)
                }
            }
            _ => {}
        }
    }
}

fn send_message(
    ui_manager: &mut UiManager,
    ui_handle: &UiHandle,
    session_server: &mut SessionClient,
) {
    let Some(textbox_text) = ui_manager.get_textbox_text(
        ui_handle,
        "message_textbox"
    ) else {
        return;
    };

    ui_manager.set_textbox_text(
        ui_handle,
        "message_textbox",
        ""
    );

    info!("Sending message: {:?}", textbox_text);

    let message = messages::GlobalChatSendMessage::new(&textbox_text);
    session_server.send_message::<channels::ClientActionsChannel, _>(&message);
}

pub fn reset_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
    // TODO: implement
}
