mod forgot_password;
mod forgot_password_finish;
mod forgot_username;
mod forgot_username_finish;
mod login;
mod register;
mod register_finish;
mod reset_password;
mod start;

use std::time::Duration;

use bevy_ecs::{
    event::{EventReader, EventWriter},
    system::{Res, ResMut},
};

use game_engine::{
    asset::{embedded_asset_event, AssetId, EmbeddedAssetEvent},
    http::HttpClient,
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    kernel::{get_querystring_param, AppExitAction},
    logging::{info, warn},
    render::components::RenderLayers,
    ui::{UiHandle, UiManager},
};

use gateway_http_proto::ResetPasswordToken;

use crate::resources::{
    BackButtonClickedEvent, ForgotPasswordButtonClickedEvent, ForgotUsernameButtonClickedEvent,
    Global, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent,
    TextboxClickedEvent,
};

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
    ResetPassword,
}

pub(crate) fn setup(
    mut global: ResMut<Global>,
    mut ui_manager: ResMut<UiManager>,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
) {
    // TODO: use some kind of catalog here?
    embedded_asset_events.send(embedded_asset_event!("../embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("../embedded/34mvvk")); // verdana icon
    embedded_asset_events.send(embedded_asset_event!("../embedded/qbgz5j")); // password eye icon

    // eye and font icon handles
    let font_icon_asset_id = AssetId::from_str("34mvvk").unwrap();
    ui_manager.set_text_icon_handle(font_icon_asset_id);

    let password_eye_icon_asset_id = AssetId::from_str("qbgz5j").unwrap();
    ui_manager.set_eye_icon_handle(password_eye_icon_asset_id);

    start::setup(
        &mut global,
        &mut ui_manager,
        &mut embedded_asset_events,
        UiKey::Start,
    );
    login::setup(
        &mut global,
        &mut ui_manager,
        &mut embedded_asset_events,
        UiKey::Login,
    );
    register::setup(
        &mut global,
        &mut ui_manager,
        &mut embedded_asset_events,
        UiKey::Register,
    );
    register_finish::setup(
        &mut global,
        &mut ui_manager,
        &mut embedded_asset_events,
        UiKey::RegisterFinish,
    );
    forgot_username::setup(
        &mut global,
        &mut ui_manager,
        &mut embedded_asset_events,
        UiKey::ForgotUsername,
    );
    forgot_username_finish::setup(
        &mut global,
        &mut ui_manager,
        &mut embedded_asset_events,
        UiKey::ForgotUsernameFinish,
    );
    forgot_password::setup(
        &mut global,
        &mut ui_manager,
        &mut embedded_asset_events,
        UiKey::ForgotPassword,
    );
    forgot_password_finish::setup(
        &mut global,
        &mut ui_manager,
        &mut embedded_asset_events,
        UiKey::ForgotPasswordFinish,
    );
    reset_password::setup(
        &mut global,
        &mut ui_manager,
        &mut embedded_asset_events,
        UiKey::ResetPassword,
    );

    // render_layer
    let layer = RenderLayers::layer(0);
    ui_manager.set_target_render_layer(layer);

    // check for reset password token on querystring
    if let Some(token) = get_querystring_param("reset_password_token") {
        info!("reset password token found in url: {}", token);
        if let Some(token) = ResetPasswordToken::from_str(&token) {
            global.reset_password_token = Some(token);
            go_to_ui(
                &mut ui_manager,
                &global,
                global.get_ui_handle(UiKey::ResetPassword),
            );
            return;
        } else {
            warn!("invalid reset password token: {}", token);
        }
    }

    if global.reset_password_token.is_none() {
        // start at start ui
        go_to_ui(&mut ui_manager, &global, global.get_ui_handle(UiKey::Start));
    }
}

pub(crate) fn handle_events(
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
        UiKey::ResetPassword => {
            reset_password::handle_events(
                &mut global,
                &mut ui_manager,
                &mut http_client,
                &mut submit_btn_rdr,
                &mut textbox_click_rdr,
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

pub(crate) fn go_to_ui(ui_manager: &mut UiManager, global: &Global, ui_handle: &UiHandle) {
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
            UiKey::ResetPassword => {
                reset_password::reset_state(ui_manager, &current_ui_handle);
            }
        }
    }

    // enable given ui
    ui_manager.enable_ui(ui_handle);
}

pub(crate) fn process_requests(
    mut global: ResMut<Global>,
    mut http_client: ResMut<HttpClient>,
    mut ui_manager: ResMut<UiManager>,
    mut app_exit_action_writer: EventWriter<AppExitAction>,
) {
    login::process_requests(
        &mut global,
        &mut http_client,
        &mut ui_manager,
        &mut app_exit_action_writer,
    );
    register::process_requests(&mut global, &mut http_client, &mut ui_manager);
    forgot_password::process_requests(&mut global, &mut http_client, &mut ui_manager);
    forgot_username::process_requests(&mut global, &mut http_client, &mut ui_manager);
    reset_password::process_requests(&mut global, &mut http_client, &mut ui_manager);
}

pub(crate) fn redirect_to_game_app(app_exit_action_writer: &mut EventWriter<AppExitAction>) {
    info!("redirecting to game app");
    app_exit_action_writer.send(AppExitAction::go_to("game"));
}

pub(crate) fn clear_spinners_if_needed(global: &Global, ui_manager: &mut UiManager) {
    if global.user_login_response_key_opt.is_none()
        && global.user_register_response_key_opt.is_none()
    {
        ui_manager.set_node_visible(&global.get_ui_handle(UiKey::Register), "spinner", false);
        ui_manager.set_node_visible(&global.get_ui_handle(UiKey::Login), "spinner", false);
        ui_manager.set_node_visible(
            &global.get_ui_handle(UiKey::ForgotUsername),
            "spinner",
            false,
        );
        ui_manager.set_node_visible(
            &global.get_ui_handle(UiKey::ForgotPassword),
            "spinner",
            false,
        );
        ui_manager.set_node_visible(
            &global.get_ui_handle(UiKey::ResetPassword),
            "spinner",
            false,
        );
    }
}
