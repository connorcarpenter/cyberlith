use bevy_ecs::event::{EventReader, EventWriter};

use game_engine::{
    asset::{embedded_asset_event, AssetId, EmbeddedAssetEvent},
    logging::info,
    ui::{UiHandle, UiManager},
};

use crate::{
    resources::{Global, SubmitButtonClickedEvent},
    ui::{go_to_ui, UiKey},
};

pub(crate) fn setup(
    global: &mut Global,
    ui_manager: &mut UiManager,
    embedded_asset_events: &mut EventWriter<EmbeddedAssetEvent>,
    ui_key: UiKey,
) {
    embedded_asset_events.send(embedded_asset_event!("../embedded/bqgxb1"));

    let ui_handle = UiHandle::new(AssetId::from_str("bqgxb1").unwrap());
    global.insert_ui(ui_key, ui_handle);
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&ui_handle, "submit_button");
}

pub(crate) fn handle_events(
    global: &mut Global,
    ui_manager: &mut UiManager,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    should_rumble: &mut bool,
) {
    // Home Button Click
    let mut submit_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_clicked = true;
    }
    if submit_clicked {
        info!("home button clicked!");
        // go to start ui
        go_to_ui(ui_manager, global, global.get_ui_handle(UiKey::Start));
        *should_rumble = true;
    }
}

pub fn reset_state(_ui_manager: &mut UiManager, _ui_handle: &UiHandle) {}
