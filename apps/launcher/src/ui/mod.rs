mod login;
mod register;
mod start;
mod register_finish;

use std::time::Duration;

use bevy_ecs::{
    event::{EventReader, EventWriter},
    system::{Res, ResMut},
};

use game_engine::{
    asset::{embedded_asset_event, AssetId, EmbeddedAssetEvent},
    http::HttpClient,
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    render::components::RenderLayers,
    ui::{UiHandle, UiManager},
};

use crate::{
    resources::{
        Global, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent,
    },
    ui::{login::ui_login_handle_events, start::ui_start_handle_events, register::ui_register_handle_events, register_finish::ui_register_finish_handle_events},
};

pub fn ui_setup(
    mut global: ResMut<Global>,
    mut ui_manager: ResMut<UiManager>,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
) {
    // TODO: use some kind of catalog here?
    embedded_asset_events.send(embedded_asset_event!("../embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("../embedded/34mvvk")); // verdana icon
    embedded_asset_events.send(embedded_asset_event!("../embedded/qbgz5j")); // password eye icon

    embedded_asset_events.send(embedded_asset_event!("../embedded/tpp7za")); // start ui
    embedded_asset_events.send(embedded_asset_event!("../embedded/3f5gej")); // login ui
    embedded_asset_events.send(embedded_asset_event!("../embedded/rckneg")); // register ui
    embedded_asset_events.send(embedded_asset_event!("../embedded/fsfn5m")); // register finish ui

    // render_layer
    let layer = RenderLayers::layer(0);
    ui_manager.set_target_render_layer(layer);

    // ui
    // TODO: use some kind of catalog?

    // start
    let start_ui_handle = UiHandle::new(AssetId::from_str("tpp7za").unwrap());
    global.ui_start_handle = Some(start_ui_handle);
    ui_manager.register_ui_event::<LoginButtonClickedEvent>(&start_ui_handle, "login_button");
    ui_manager.register_ui_event::<RegisterButtonClickedEvent>(&start_ui_handle, "register_button");

    // login
    let login_ui_handle = UiHandle::new(AssetId::from_str("3f5gej").unwrap());
    global.ui_login_handle = Some(login_ui_handle);
    ui_manager.register_ui_event::<RegisterButtonClickedEvent>(&login_ui_handle, "register_button");
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&login_ui_handle, "submit_button");

    // register
    let register_ui_handle = UiHandle::new(AssetId::from_str("rckneg").unwrap());
    global.ui_register_handle = Some(register_ui_handle);
    ui_manager.register_ui_event::<LoginButtonClickedEvent>(&register_ui_handle, "login_button");
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&register_ui_handle, "submit_button");

    // register_finish
    let register_finish_ui_handle = UiHandle::new(AssetId::from_str("fsfn5m").unwrap());
    global.ui_register_finish_handle = Some(register_finish_ui_handle);
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&register_finish_ui_handle, "submit_button");

    // other config
    ui_manager.enable_ui(&start_ui_handle);
}

pub fn ui_handle_events(
    mut global: ResMut<Global>,
    input: Res<Input>,
    mut ui_manager: ResMut<UiManager>,
    mut http_client: ResMut<HttpClient>,
    mut rumble_manager: ResMut<RumbleManager>,

    mut login_btn_rdr: EventReader<LoginButtonClickedEvent>,
    mut register_btn_rdr: EventReader<RegisterButtonClickedEvent>,
    mut submit_btn_rdr: EventReader<SubmitButtonClickedEvent>,
) {
    let current_ui_handle = ui_manager.active_ui();

    let mut should_rumble = false;

    if current_ui_handle == global.ui_start_handle {
        ui_start_handle_events(
            &mut ui_manager,
            &global,
            &mut login_btn_rdr,
            &mut register_btn_rdr,
            &mut submit_btn_rdr,
            &mut should_rumble,
        );
    } else if current_ui_handle == global.ui_login_handle {
        ui_login_handle_events(
            &mut global,
            &mut ui_manager,
            &mut http_client,
            &mut login_btn_rdr,
            &mut register_btn_rdr,
            &mut submit_btn_rdr,
            &mut should_rumble,
        );
    } else if current_ui_handle == global.ui_register_handle {
        ui_register_handle_events(
            &mut global,
            &mut ui_manager,
            &mut http_client,
            &mut login_btn_rdr,
            &mut register_btn_rdr,
            &mut submit_btn_rdr,
            &mut should_rumble,
        );
    } else if current_ui_handle == global.ui_register_finish_handle {
        ui_register_finish_handle_events(
            &mut global,
            &mut ui_manager,
            &mut login_btn_rdr,
            &mut register_btn_rdr,
            &mut submit_btn_rdr,
            &mut should_rumble,
        );
    }

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