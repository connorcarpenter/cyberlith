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

use crate::resources::{Global, LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent, TextboxClickedEvent};

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
    ui_manager.register_ui_event::<TextboxClickedEvent>(&login_ui_handle, "username_textbox");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&login_ui_handle, "password_textbox");

    // register
    let register_ui_handle = UiHandle::new(AssetId::from_str("rckneg").unwrap());
    global.ui_register_handle = Some(register_ui_handle);
    ui_manager.register_ui_event::<LoginButtonClickedEvent>(&register_ui_handle, "login_button");
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&register_ui_handle, "submit_button");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&register_ui_handle, "username_textbox");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&register_ui_handle, "email_textbox");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&register_ui_handle, "password_textbox");
    ui_manager.register_ui_event::<TextboxClickedEvent>(&register_ui_handle, "confirm_password_textbox");

    // register_finish
    let register_finish_ui_handle = UiHandle::new(AssetId::from_str("fsfn5m").unwrap());
    global.ui_register_finish_handle = Some(register_finish_ui_handle);
    ui_manager.register_ui_event::<SubmitButtonClickedEvent>(&register_finish_ui_handle, "submit_button");

    // other config
    go_to_ui(&mut ui_manager, &global, &start_ui_handle);
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
    mut textbox_click_rdr: EventReader<TextboxClickedEvent>,
) {
    let current_ui_handle = ui_manager.active_ui();

    let mut should_rumble = false;

    if current_ui_handle == global.ui_start_handle {
        start::handle_events(
            &mut ui_manager,
            &global,
            &mut login_btn_rdr,
            &mut register_btn_rdr,
            &mut submit_btn_rdr,
            &mut textbox_click_rdr,
            &mut should_rumble,
        );
    } else if current_ui_handle == global.ui_login_handle {
        login::handle_events(
            &mut global,
            &mut ui_manager,
            &mut http_client,
            &mut login_btn_rdr,
            &mut register_btn_rdr,
            &mut submit_btn_rdr,
            &mut textbox_click_rdr,
            &mut should_rumble,
        );
    } else if current_ui_handle == global.ui_register_handle {
        register::handle_events(
            &mut global,
            &mut ui_manager,
            &mut http_client,
            &mut login_btn_rdr,
            &mut register_btn_rdr,
            &mut submit_btn_rdr,
            &mut textbox_click_rdr,
            &mut should_rumble,
        );
    } else if current_ui_handle == global.ui_register_finish_handle {
        register_finish::handle_events(
            &mut global,
            &mut ui_manager,
            &mut login_btn_rdr,
            &mut register_btn_rdr,
            &mut submit_btn_rdr,
            &mut textbox_click_rdr,
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

pub(crate) fn go_to_ui(
    ui_manager: &mut UiManager,
    global: &Global,
    ui_handle: &UiHandle,
) {
    let current_ui_handle = ui_manager.active_ui();

    // cleanup ui that we are disabling
    if current_ui_handle == global.ui_start_handle {

        // nothing to clean up

    } else if current_ui_handle == global.ui_login_handle {

        let ui_handle = global.ui_login_handle.unwrap();

        // clear textboxes
        ui_manager.set_text(&ui_handle, "username_textbox", "");
        ui_manager.set_text(&ui_handle, "password_textbox", "");
        ui_manager.set_textbox_password_eye_visible(&ui_handle, "password_textbox", false);

        // clear error output
        ui_manager.set_text(&ui_handle, "error_output_text", "");

        // clear spinner
        ui_manager.set_node_visible(&ui_handle, "spinner", false);

    } else if current_ui_handle == global.ui_register_handle {

        let ui_handle = global.ui_register_handle.unwrap();

        // clear textboxes
        ui_manager.set_text(&ui_handle, "username_textbox", "");
        ui_manager.set_text(&ui_handle, "email_textbox", "");
        ui_manager.set_text(&ui_handle, "password_textbox", "");
        ui_manager.set_textbox_password_eye_visible(&ui_handle, "password_textbox", false);
        ui_manager.set_text(&ui_handle, "confirm_password_textbox", "");
        ui_manager.set_textbox_password_eye_visible(&ui_handle, "confirm_password_textbox", false);

        // clear error output
        ui_manager.set_text(&ui_handle, "error_output_text", "");

        // clear spinner
        ui_manager.set_node_visible(&ui_handle, "spinner", false);

    } else if current_ui_handle == global.ui_register_finish_handle {

        // nothing to clean up

    }

    // enable given ui
    ui_manager.enable_ui(ui_handle);
}