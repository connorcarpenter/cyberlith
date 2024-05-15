
use bevy_ecs::event::{EventReader, EventWriter};

use game_engine::{
    asset::{AssetId, embedded_asset_event, EmbeddedAssetEvent},
    http::HttpClient,
    ui::{UiManager, UiHandle},
};

use crate::{resources::{
    Global, TextboxClickedEvent, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent, BackButtonClickedEvent,
}, ui::UiKey};

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
    http_client: &mut HttpClient,
    login_btn_rdr: &mut EventReader<LoginButtonClickedEvent>,
    register_btn_rdr: &mut EventReader<RegisterButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    textbox_click_rdr: &mut EventReader<TextboxClickedEvent>,
    should_rumble: &mut bool,
) {

}

pub fn reset_state(
    ui_manager: &mut UiManager,
    ui_handle: &UiHandle
) {

}
