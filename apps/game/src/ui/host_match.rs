use bevy_ecs::event::EventReader;

use game_engine::{
    logging::info,
    ui::{UiHandle, UiManager},
};

use crate::ui::{events::SubmitButtonClickedEvent, UiCatalog, UiKey};

pub(crate) fn on_load(ui_catalog: &mut UiCatalog, ui_manager: &mut UiManager) {
    let ui_key = UiKey::HostMatch;
    let ui_handle = ui_catalog.get_ui_handle(ui_key);

    ui_catalog.set_loaded(ui_key);
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&ui_handle, "submit_button");
}

pub(crate) fn handle_events(
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    should_rumble: &mut bool,
) {
    // Submit Button Click
    let mut submit_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_clicked = true;
    }
    if submit_clicked {
        info!("submit button clicked!");
        *should_rumble = true;
    }
}

pub fn reset_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {
    // TODO: implement
}
