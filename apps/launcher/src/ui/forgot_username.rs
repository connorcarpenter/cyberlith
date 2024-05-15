
use bevy_ecs::event::{EventReader, EventWriter};

use game_engine::{
    asset::{AssetId, embedded_asset_event, EmbeddedAssetEvent},
    http::HttpClient,
    ui::{UiManager, UiHandle},
    logging::info,
};

use crate::{resources::{
    Global, TextboxClickedEvent, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent, BackButtonClickedEvent,
}, ui::{UiKey, go_to_ui}};

pub(crate) fn setup(
    global: &mut Global,
    ui_manager: &mut UiManager,
    embedded_asset_events: &mut EventWriter<EmbeddedAssetEvent>,
    ui_key: UiKey,
) {
    embedded_asset_events.send(embedded_asset_event!("../embedded/tksh5u"));

    let ui_handle = UiHandle::new(AssetId::from_str("tksh5u").unwrap());
    global.insert_ui(ui_key, ui_handle);
    ui_manager.register_ui_event::<BackButtonClickedEvent>(&ui_handle, "back_button");
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&ui_handle, "submit_button");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&ui_handle, "email_textbox");
}

pub(crate) fn handle_events(
    global: &mut Global,
    ui_manager: &mut UiManager,
    back_btn_rdr: &mut EventReader<BackButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    should_rumble: &mut bool,
) {
    // Back Button Click
    let mut back_btn_clicked = false;
    for _ in back_btn_rdr.read() {
        back_btn_clicked = true;
    }
    if back_btn_clicked {
        info!("back button clicked!");
        go_to_ui(ui_manager, global, &global.get_ui_handle(UiKey::Login));
        *should_rumble = true;
    }

    // Submit Button Click
    let mut submit_btn_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_btn_clicked = true;
    }
    if submit_btn_clicked {
        info!("submit button clicked!");

        // TODO: send request to backend

        *should_rumble = true;
    }
}

pub fn reset_state(
    ui_manager: &mut UiManager,
    ui_handle: &UiHandle
) {
    // clear textboxes
    ui_manager.set_text(&ui_handle, "email_textbox", "");

    // clear error output
    ui_manager.set_text(&ui_handle, "error_output_text", "");

    // clear spinner
    ui_manager.set_node_visible(&ui_handle, "spinner", false);
}
