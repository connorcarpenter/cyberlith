
use game_engine::ui::{UiHandle, UiManager};

use crate::ui::{go_to_sub_ui, UiCatalog, UiKey};

pub(crate) fn on_load(
    ui_catalog: &mut UiCatalog,
    ui_manager: &mut UiManager,
) {
    let ui_key = UiKey::GlobalChat;

    ui_catalog.set_loaded(ui_key);

    if let Some(active_ui_handle) = ui_manager.active_ui() {
        if ui_catalog.get_ui_key(&active_ui_handle) == UiKey::MainMenu {
            go_to_sub_ui(ui_manager, ui_catalog, UiKey::GlobalChat);
        }
    }
}

pub(crate) fn handle_events(
    // enter key maybe?
    _should_rumble: &mut bool,
) {

}

pub fn reset_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
    // TODO: implement
}
