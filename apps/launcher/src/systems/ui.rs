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

fn ui_start_handle_events(
    ui_manager: &mut UiManager,
    global: &Global,
    login_btn_rdr: &mut EventReader<LoginButtonClickedEvent>,
    register_btn_rdr: &mut EventReader<RegisterButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    should_rumble: &mut bool,
) {
    // in Start Ui

    // Login Button Click
    let mut login_clicked = false;
    for _ in login_btn_rdr.read() {
        login_clicked = true;
    }
    if login_clicked {
        info!("login button clicked!");
        ui_manager.enable_ui(&global.ui_login_handle.unwrap());
        *should_rumble = true;
    }

    // Register Button Click
    let mut register_clicked = false;
    for _ in register_btn_rdr.read() {
        register_clicked = true;
    }
    if register_clicked {
        info!("register button clicked!");
        ui_manager.enable_ui(&global.ui_register_handle.unwrap());
        *should_rumble = true;
    }

    // drain others
    for _ in submit_btn_rdr.read() {
        // ignore
    }
}

fn ui_register_handle_events(
    global: &mut Global,
    ui_manager: &mut UiManager,
    http_client: &mut HttpClient,
    login_btn_rdr: &mut EventReader<LoginButtonClickedEvent>,
    register_btn_rdr: &mut EventReader<RegisterButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    should_rumble: &mut bool,
) {
    // in Register Ui

    // Login Button Click
    let mut login_clicked = false;
    for _ in login_btn_rdr.read() {
        login_clicked = true;
    }
    if login_clicked {
        info!("login button clicked!");

        // TODO: validate!

        ui_manager.enable_ui(&global.ui_login_handle.unwrap());
        *should_rumble = true;
    }

    // Submit Button Click
    let mut submit_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_clicked = true;
    }
    if submit_clicked {
        info!("submit button clicked!");

        // TODO: validate!

        backend_send_register_request(global, ui_manager, http_client);

        *should_rumble = true;
    }

    // drain others
    for _ in register_btn_rdr.read() {
        // ignore
    }
}

fn ui_register_finish_handle_events(
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

fn ui_login_handle_events(
    global: &mut Global,
    ui_manager: &mut UiManager,
    http_client: &mut HttpClient,
    login_btn_rdr: &mut EventReader<LoginButtonClickedEvent>,
    register_btn_rdr: &mut EventReader<RegisterButtonClickedEvent>,
    submit_btn_rdr: &mut EventReader<SubmitButtonClickedEvent>,
    should_rumble: &mut bool,
) {
    // in Login Ui

    // Register Button Click
    let mut register_clicked = false;
    for _ in register_btn_rdr.read() {
        register_clicked = true;
    }
    if register_clicked {
        info!("register button clicked!");
        ui_manager.enable_ui(&global.ui_register_handle.unwrap());
        *should_rumble = true;
    }

    // Submit Button Click
    let mut submit_clicked = false;
    for _ in submit_btn_rdr.read() {
        submit_clicked = true;
    }
    if submit_clicked {
        info!("submit button clicked!");

        backend_send_login_request(global, ui_manager, http_client);

        *should_rumble = true;
    }

    // drain others
    for _ in login_btn_rdr.read() {
        // ignore
    }
}
