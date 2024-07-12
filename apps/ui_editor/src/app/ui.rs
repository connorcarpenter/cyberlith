use std::time::Duration;

use bevy_ecs::{
    event::{Event, EventReader, EventWriter},
    prelude::Commands,
    system::{Res, ResMut},
};

use game_engine::{
    asset::{
        AssetId, AssetManager, embedded_asset_event, EmbeddedAssetEvent,
    },
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    render::components::{
        Camera, CameraBundle, ClearOperation, OrthographicProjection, Projection, RenderLayers,
        RenderTarget,
    },
    ui::UiManager,
};

use logging::info;

use crate::app::{global::Global, uis::*, examples::{setup_user_list_test_case, setup_global_chat_test_case}};

#[derive(Event, Default)]
pub struct SubmitButtonEvent;

// this is run as a system at startup
pub fn setup(
    mut commands: Commands,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
) {
    // camera
    let scene_camera_entity = commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: None,
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Screen,
                ..Default::default()
            },
            projection: Projection::Orthographic(OrthographicProjection {
                near: 0.0,
                far: 2000.0,
            }),
            ..Default::default()
        })
        .insert(RenderLayers::layer(0))
        .id();

    let mut global = Global::new(scene_camera_entity);

    // ui setup

    // global.load_ui(ui_manager, launcher::start::ui_define()); // start
    // global.load_ui(ui_manager, launcher::login::ui_define()); // login
    // global.load_ui(ui_manager, launcher::register::ui_define()); // register
    // global.load_ui(ui_manager, launcher::register_finish::ui_define()); // register_finish
    // global.load_ui(ui_manager, launcher::forgot_username::ui_define()); // forgot username
    // global.load_ui(ui_manager, launcher::forgot_username_finish::ui_define()); // forgot username finish
    // global.load_ui(ui_manager, launcher::forgot_password::ui_define()); // forgot password
    // global.load_ui(ui_manager, launcher::forgot_password_finish::ui_define()); // forgot password finish
    // global.load_ui(ui_manager, launcher::reset_password::ui_define()); // reset password

    let main_menu_ui_handle = global.load_ui(&mut ui_manager, game::main_menu::ui_define()); // game main menu

    // global.load_ui(&mut ui_manager, game::host_match::ui_define()); // game host match

    ui_manager.set_target_render_layer(RenderLayers::layer(0));
    ui_manager.enable_ui(&main_menu_ui_handle);

    // global chat
    let global_chat_state = setup_global_chat_test_case(
        &mut global,
        &mut ui_manager,
        &asset_manager,
        &main_menu_ui_handle,
    );
    commands.insert_resource(global_chat_state);

    // user list
    let user_list_state = setup_user_list_test_case(
        &mut global,
        &mut ui_manager,
        &asset_manager,
        &main_menu_ui_handle,
    );
    commands.insert_resource(user_list_state);

    // ui setup
    embedded_asset_events.send(embedded_asset_event!("embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("embedded/34mvvk")); // verdana icon
    embedded_asset_events.send(embedded_asset_event!("embedded/qbgz5j")); // password eye icon

    // font & password eye icon setup
    ui_manager.set_text_icon_handle(AssetId::from_str("34mvvk").unwrap());
    ui_manager.set_eye_icon_handle(AssetId::from_str("qbgz5j").unwrap());

    commands.insert_resource(global);
}

pub fn handle_events(
    input: Res<Input>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut login_btn_rdr: EventReader<SubmitButtonEvent>,
) {
    for _ in login_btn_rdr.read() {
        info!("login button clicked!");
        if let Some(id) = input.gamepad_first() {
            rumble_manager.add_rumble(
                id,
                Duration::from_millis(200),
                GamepadRumbleIntensity::weak_motor(0.4),
            );
        }
    }
}
