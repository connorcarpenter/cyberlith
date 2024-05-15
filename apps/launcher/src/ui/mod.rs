mod login;
mod register;
mod start;
mod register_finish;
mod forgot_username;
mod forgot_username_finish;
mod forgot_password;
mod forgot_password_finish;

use std::time::Duration;

use bevy_ecs::{
    event::{EventReader, EventWriter},
    system::{Res, ResMut},
};

use game_engine::{
    asset::{embedded_asset_event, EmbeddedAssetEvent},
    http::HttpClient,
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    render::components::RenderLayers,
    ui::{UiHandle, UiManager},
};

use crate::resources::{BackButtonClickedEvent, ForgotPasswordButtonClickedEvent, ForgotUsernameButtonClickedEvent, Global, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent, TextboxClickedEvent};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UiKey {
    Start,
    Login,
    Register,
    RegisterFinish,
    ForgotUsername,
    ForgotUsernameFinish,
    ForgotPassword,
    ForgotPasswordFinish,
}

pub fn ui_setup(
    mut global: ResMut<Global>,
    mut ui_manager: ResMut<UiManager>,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
) {
    // TODO: use some kind of catalog here?
    embedded_asset_events.send(embedded_asset_event!("../embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("../embedded/34mvvk")); // verdana icon
    embedded_asset_events.send(embedded_asset_event!("../embedded/qbgz5j")); // password eye icon

    start::setup(&mut global, &mut ui_manager, &mut embedded_asset_events, UiKey::Start);
    login::setup(&mut global, &mut ui_manager, &mut embedded_asset_events, UiKey::Login);
    register::setup(&mut global, &mut ui_manager, &mut embedded_asset_events, UiKey::Register);
    register_finish::setup(&mut global, &mut ui_manager, &mut embedded_asset_events, UiKey::RegisterFinish);
    forgot_username::setup(&mut global, &mut ui_manager, &mut embedded_asset_events, UiKey::ForgotUsername);
    forgot_username_finish::setup(&mut global, &mut ui_manager, &mut embedded_asset_events, UiKey::ForgotUsernameFinish);
    forgot_password::setup(&mut global, &mut ui_manager, &mut embedded_asset_events, UiKey::ForgotPassword);
    forgot_password_finish::setup(&mut global, &mut ui_manager, &mut embedded_asset_events, UiKey::ForgotPasswordFinish);

    // render_layer
    let layer = RenderLayers::layer(0);
    ui_manager.set_target_render_layer(layer);

    // other config
    go_to_ui(&mut ui_manager, &global, global.get_ui_handle(UiKey::Start));
}

pub fn ui_handle_events(
    mut global: ResMut<Global>,
    input: Res<Input>,
    mut ui_manager: ResMut<UiManager>,
    mut http_client: ResMut<HttpClient>,
    mut rumble_manager: ResMut<RumbleManager>,

    mut login_btn_rdr: EventReader<LoginButtonClickedEvent>,
    mut register_btn_rdr: EventReader<RegisterButtonClickedEvent>,
    mut back_btn_rdr: EventReader<BackButtonClickedEvent>,
    mut submit_btn_rdr: EventReader<SubmitButtonClickedEvent>,
    mut forgot_username_btn_rdr: EventReader<ForgotUsernameButtonClickedEvent>,
    mut forgot_password_btn_rdr: EventReader<ForgotPasswordButtonClickedEvent>,
    mut textbox_click_rdr: EventReader<TextboxClickedEvent>,
) {
    let Some(current_ui_handle) = ui_manager.active_ui() else {
        return;
    };

    let mut should_rumble = false;

    match global.get_ui_key(&current_ui_handle) {
        UiKey::Start => {
            start::handle_events(
                &mut ui_manager,
                &global,
                &mut login_btn_rdr,
                &mut register_btn_rdr,
                &mut should_rumble,
            );
        }
        UiKey::Login => {
            login::handle_events(
                &mut global,
                &mut ui_manager,
                &mut http_client,
                &mut register_btn_rdr,
                &mut submit_btn_rdr,
                &mut forgot_username_btn_rdr,
                &mut forgot_password_btn_rdr,
                &mut textbox_click_rdr,
                &mut should_rumble,
            );
        }
        UiKey::Register => {
            register::handle_events(
                &mut global,
                &mut ui_manager,
                &mut http_client,
                &mut login_btn_rdr,
                &mut submit_btn_rdr,
                &mut textbox_click_rdr,
                &mut should_rumble,
            );
        }
        UiKey::RegisterFinish => {
            register_finish::handle_events(
                &mut global,
                &mut ui_manager,
                &mut submit_btn_rdr,
                &mut should_rumble,
            );
        }
        UiKey::ForgotUsername => {
            forgot_username::handle_events(
                &mut global,
                &mut ui_manager,
                &mut http_client,
                &mut back_btn_rdr,
                &mut submit_btn_rdr,
                &mut textbox_click_rdr,
                &mut should_rumble,
            );
        }
        UiKey::ForgotUsernameFinish => {
            forgot_username_finish::handle_events(
                &mut global,
                &mut ui_manager,
                &mut submit_btn_rdr,
                &mut should_rumble,
            );
        }
        UiKey::ForgotPassword => {
            forgot_password::handle_events(
                &mut global,
                &mut ui_manager,
                &mut http_client,
                &mut back_btn_rdr,
                &mut submit_btn_rdr,
                &mut textbox_click_rdr,
                &mut should_rumble,
            );
        }
        UiKey::ForgotPasswordFinish => {
            forgot_password_finish::handle_events(
                &mut global,
                &mut ui_manager,
                &mut submit_btn_rdr,
                &mut should_rumble,
            );
        }
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

    // drain all events
    for _ in login_btn_rdr.read() {}
    for _ in register_btn_rdr.read() {}
    for _ in submit_btn_rdr.read() {}
    for _ in forgot_username_btn_rdr.read() {}
    for _ in forgot_password_btn_rdr.read() {}
    for _ in textbox_click_rdr.read() {}
}

pub(crate) fn go_to_ui(
    ui_manager: &mut UiManager,
    global: &Global,
    ui_handle: &UiHandle,
) {
    if let Some(current_ui_handle) = ui_manager.active_ui() {
        match global.get_ui_key(&current_ui_handle) {
            UiKey::Start => {
                start::reset_state(ui_manager, &current_ui_handle);
            }
            UiKey::Login => {
                login::reset_state(ui_manager, &current_ui_handle);
            }
            UiKey::Register => {
                register::reset_state(ui_manager, &current_ui_handle);
            }
            UiKey::RegisterFinish => {
                register_finish::reset_state(ui_manager, &current_ui_handle);
            }
            UiKey::ForgotUsername => {
                forgot_username::reset_state(ui_manager, &current_ui_handle);
            }
            UiKey::ForgotUsernameFinish => {
                forgot_username_finish::reset_state(ui_manager, &current_ui_handle);
            }
            UiKey::ForgotPassword => {
                forgot_password::reset_state(ui_manager, &current_ui_handle);
            }
            UiKey::ForgotPasswordFinish => {
                forgot_password_finish::reset_state(ui_manager, &current_ui_handle);
            }
        }
    }

    // enable given ui
    ui_manager.enable_ui(ui_handle);
}