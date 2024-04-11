use bevy_app::{App, Startup, Update};
use bevy_ecs::change_detection::ResMut;
use bevy_log::info;

use game_engine::{
    render::{resources::WindowSettings, Draw},
    wait_for_finish, EnginePlugin,
    http::HttpClient,
    config::{GATEWAY_PORT, PUBLIC_IP_ADDR},
};

use crate::app::{
    resources::Global,
    systems::{
        draw, resize, scene, ui,
    },
};

pub fn run() {
    let mut app = App::default();

    app.add_plugins(EnginePlugin)
        // Add Window Settings Plugin
        .insert_resource(WindowSettings {
            title: "Cyberlith Launcher".to_string(),
            min_size: (320, 180),
            max_size: None,
            ..Default::default()
        })
        // global resource
        .init_resource::<Global>()
        // events

        .add_event::<LoginButtonClickedEvent>()
        .add_event::<RegisterButtonClickedEvent>()
        .add_event::<SubmitButtonClickedEvent>()
        // ui systems
        .add_systems(Startup, ui::ui_setup)
        .add_systems(Update, ui::ui_handle_events)
        // scene systems
        .add_systems(Startup, scene::scene_setup)
        .add_systems(Update, scene::scene_step)
        //.add_systems(Update, gamepad_testing::gamepad_testing_system)
        // viewport resize
        .add_systems(Update, resize::handle_viewport_resize)
        // draw
        .add_systems(Draw, draw::draw)

        // test gateway request
        .add_systems(Startup, test_request)
        .add_systems(Update, test_request_process)
    ;
    app.run();
}

#[allow(dead_code)]
pub async fn finished() {
    wait_for_finish().await;
}

use gateway_http_proto::UserPasswordResetRequest;
use crate::app::resources::{LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent};

// used as a system
pub fn test_request(
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    // user register
    // let request = UserRegisterRequest::new("connorc", "connorcarpenter@gmail.com", "12345");
    // let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);
    // global.user_register_response_key_opt = Some(key);
    // info!("client -> gateway: (UserRegisterRequest)");

    // user register confirm
    // let request = UserRegisterConfirmRequest::new("1vnv1v");
    // let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);
    // global.user_register_confirm_response_key_opt = Some(key);
    // info!("client -> gateway: (UserRegisterConfirmRequest)");

    // user name forgot
    // let request = UserNameForgotRequest::new("c@gmail.com");
    // let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);

    // user password forgot
    // let request = UserPasswordForgotRequest::new("connorcarpenter@gmail.com");
    // let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);
    // global.user_password_forgot_response_key_opt = Some(key);
    // info!("client -> gateway: (UserForgotPasswordRequest)");

    // user password reset
    let request = UserPasswordResetRequest::new("avpc7u", "my_cool_new_password");
    let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);
    global.user_password_reset_response_key_opt = Some(key);
    info!("client -> gateway: (UserResetPasswordRequest)");
}

fn test_request_process(
    mut http_client: ResMut<HttpClient>,
    mut global: ResMut<Global>,
) {
    if global.user_register_response_key_opt.is_some() {
        let Some(key) = &global.user_register_response_key_opt else {
            return;
        };
        let Some(result) = http_client.recv(key) else {
            return;
        };
        match result {
            Ok(_response) => {
                info!("client <- gateway: (UserRegisterResponse - 200 OK)");
                global.user_register_response_key_opt = None;
            }
            Err(err) => {
                info!("client <- gateway: (UserRegisterResponse - ERROR! {})", err.to_string());
            }
        }
    }
    if global.user_register_confirm_response_key_opt.is_some() {
        let Some(key) = &global.user_register_confirm_response_key_opt else {
            return;
        };
        let Some(result) = http_client.recv(key) else {
            return;
        };
        match result {
            Ok(_response) => {
                info!("client <- gateway: (UserRegisterConfirmResponse - 200 OK)");
                global.user_register_confirm_response_key_opt = None;
            }
            Err(err) => {
                info!("client <- gateway: (UserRegisterConfirmResponse - ERROR! {})", err.to_string());
            }
        }
    }
    if global.user_password_forgot_response_key_opt.is_some() {
        let Some(key) = &global.user_password_forgot_response_key_opt else {
            return;
        };
        let Some(result) = http_client.recv(key) else {
            return;
        };
        match result {
            Ok(_response) => {
                info!("client <- gateway: (UserPasswordForgotResponse - 200 OK)");
                global.user_password_forgot_response_key_opt = None;
            }
            Err(err) => {
                info!("client <- gateway: (UserPasswordForgotResponse - ERROR! {})", err.to_string());
            }
        }
    }
    if global.user_password_reset_response_key_opt.is_some() {
        let Some(key) = &global.user_password_reset_response_key_opt else {
            return;
        };
        let Some(result) = http_client.recv(key) else {
            return;
        };
        match result {
            Ok(_response) => {
                info!("client <- gateway: (UserPasswordResetResponse - 200 OK)");
                global.user_password_reset_response_key_opt = None;
            }
            Err(err) => {
                info!("client <- gateway: (UserPasswordResetResponse - ERROR! {})", err.to_string());
            }
        }
    }
}