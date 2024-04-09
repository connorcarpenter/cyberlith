use std::time::Duration;

use bevy_ecs::{
    event::{Event, EventReader, EventWriter},
    system::{Commands, Res, ResMut},
};
use bevy_log::info;

use game_engine::{
    asset::{
        embedded_asset_event, AssetId, EmbeddedAssetEvent,
    },
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    render::{
        base::Color,
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation, OrthographicProjection, Projection,
            RenderLayers, RenderTarget,
        },
    },
    ui::{UiHandle, UiManager},
};

use crate::app::resources::Global;

#[derive(Event, Default)]
pub struct StartButtonEvent;

#[derive(Event, Default)]
pub struct ContinueButtonEvent;

pub fn ui_setup(
    mut commands: Commands,
    mut global: ResMut<Global>,
    mut ui_manager: ResMut<UiManager>,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
) {
    // TODO: use some kind of catalog here?
    embedded_asset_events.send(embedded_asset_event!("../embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("../embedded/34mvvk")); // verdana icon

    // embedded_asset_events.send(embedded_asset_event!("../embedded/tpp7za")); // start ui
    // embedded_asset_events.send(embedded_asset_event!("../embedded/?")); // login ui
    embedded_asset_events.send(embedded_asset_event!("../embedded/rckneg")); // register ui

    // render_layer
    let layer = RenderLayers::layer(1);

    // ui
    // TODO: use some kind of catalog?
    // let _start_ui_handle = UiHandle::new(AssetId::from_str("tpp7za").unwrap());
    // let _login_ui_handle = UiHandle::new(AssetId::from_str("rckneg").unwrap());
    let register_ui_handle = UiHandle::new(AssetId::from_str("rckneg").unwrap());
    ui_manager.set_target_render_layer(layer);
    ui_manager.enable_ui(&register_ui_handle);

    //asset_manager.register_ui_event::<StartButtonEvent>(&ui_handle, "login_button");
    //asset_manager.register_ui_event::<ContinueButtonEvent>(&ui_handle, "register_button");

    // light
    commands
        .spawn(AmbientLight::new(1.0, Color::WHITE))
        .insert(layer);

    // camera
    let camera_id = commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: None, // this will set later
                target: RenderTarget::Screen,
                clear_operation: ClearOperation::none(),
                ..Default::default()
            },
            projection: Projection::Orthographic(OrthographicProjection {
                near: 0.0,
                far: 2000.0,
            }),
            ..Default::default()
        })
        .insert(layer)
        .id();

    global.camera_ui = camera_id;
}

pub fn ui_handle_events(
    input: Res<Input>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut login_btn_rdr: EventReader<StartButtonEvent>,
    mut register_btn_rdr: EventReader<ContinueButtonEvent>,
) {
    for _ in login_btn_rdr.read() {
        info!("login button clicked!");
        if let Some(id) = input.gamepad_first() {
            rumble_manager.add_rumble(
                id,
                Duration::from_millis(200),
                GamepadRumbleIntensity::strong_motor(0.4),
            );
        }
    }
    for _ in register_btn_rdr.read() {
        info!("register button clicked!");
        if let Some(id) = input.gamepad_first() {
            rumble_manager.add_rumble(
                id,
                Duration::from_millis(200),
                GamepadRumbleIntensity::strong_motor(0.4),
            );
        }
    }
}
