
use game_engine::ui::{UiHandle, UiManager};

use crate::ui::{UiCatalog, UiKey};

pub(crate) fn on_load(
    ui_catalog: &mut UiCatalog,
) {
    let ui_key = UiKey::GlobalChat;
    let ui_handle = UiHandle::new(UiCatalog::game_global_chat_ui());

    ui_catalog.insert_ui(ui_key, ui_handle);
}

pub(crate) fn handle_events(
    // enter key maybe?
    _should_rumble: &mut bool,
) {

}

pub fn reset_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
    // TODO: implement
}
