use std::time::Duration;

use bevy_ecs::{
    event::{EventReader, EventWriter},
    system::{Res, ResMut},
};

use game_engine::{
    asset::{embedded_asset_event, AssetId, EmbeddedAssetEvent},
    http::HttpClient,
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    logging::info,
    render::components::RenderLayers,
    ui::{UiHandle, UiManager},
};

use crate::{
    resources::{
        Global, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent,
    },
    systems::backend::{backend_send_login_request, backend_send_register_request},
};

pub(crate) fn ui_register_finish_handle_events(
    global: &mut Global,
    ui_manager: &mut UiManager,
    login_btn_rdr: &mut EventReader<LoginButtonClickedEvent>,
    register_btn_rdr: &mut EventReader<RegisterButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    should_rumble: &mut bool,
) {
    // in Register Finish Ui

    // Submit Button Click
    let mut submit_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_clicked = true;
    }
    if submit_clicked {
        info!("home button clicked!");
        // go to start ui
        ui_manager.enable_ui(&global.ui_start_handle.unwrap());
        *should_rumble = true;
    }

    // drain others
    for _ in login_btn_rdr.read() {
        // ignore
    }
    for _ in register_btn_rdr.read() {
        // ignore
    }
}