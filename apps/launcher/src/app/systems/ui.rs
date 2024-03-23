use std::time::Duration;

use bevy_ecs::{
    event::{Event, EventReader, EventWriter},
    system::{Commands, Res, ResMut, Query},
};
use bevy_log::info;

use game_engine::{
    input::{InputEvent, Input, GamepadRumbleIntensity, RumbleManager},
    asset::{embedded_asset_event, AssetHandle, AssetId, AssetManager, EmbeddedAssetEvent, UiData},
    render::{
        base::Color,
        components::{
            RenderLayer, AmbientLight, Camera, CameraBundle, ClearOperation, OrthographicProjection, Projection,
            RenderLayers, RenderTarget,
        },
    },
    ui::{UiInputConverter},
};

use crate::app::resources::Global;

#[derive(Event, Default)]
pub struct StartButtonEvent;

#[derive(Event, Default)]
pub struct ContinueButtonEvent;

pub fn ui_setup(
    mut commands: Commands,
    mut global: ResMut<Global>,
    mut asset_manager: ResMut<AssetManager>,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
) {
    // TODO: use some kind of catalog here?
    embedded_asset_events.send(embedded_asset_event!("../embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("../embedded/34mvvk")); // verdana icon
    embedded_asset_events.send(embedded_asset_event!("../embedded/tpp7za")); // main ui

    // render_layer
    let layer = RenderLayers::layer(1);

    // ui
    let ui_handle = AssetHandle::<UiData>::new(AssetId::from_str("tpp7za").unwrap()); // TODO: use some kind of catalog?
    let _ui_entity = commands.spawn(ui_handle).insert(layer).id();

    asset_manager.register_ui_event::<StartButtonEvent>(&ui_handle, "start_button");
    asset_manager.register_ui_event::<ContinueButtonEvent>(&ui_handle, "continue_button");

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

pub fn ui_update(
    mut asset_manager: ResMut<AssetManager>,
    input: Res<Input>,
    mut input_events: EventReader<InputEvent>,
    // Cameras
    cameras_q: Query<(&Camera, Option<&RenderLayer>)>,
    // UIs
    uis_q: Query<(&AssetHandle<UiData>, Option<&RenderLayer>)>,
) {
    let ui_input = UiInputConverter::convert(&input, &mut input_events);

    for (ui_handle, ui_render_layer_opt) in uis_q.iter() {

        // find camera, update viewport
        for (camera, cam_render_layer_opt) in cameras_q.iter() {
            if cam_render_layer_opt == ui_render_layer_opt {
                asset_manager.update_ui_viewport(camera, ui_handle);
                break;
            }
        }

        // update with inputs
        let Some(ui_input) = ui_input.clone() else {
            continue;
        };
        asset_manager.update_ui_input(ui_input, ui_handle);
    }
}

pub fn ui_handle_events(
    input: Res<Input>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut start_btn_rdr: EventReader<StartButtonEvent>,
    mut continue_btn_rdr: EventReader<ContinueButtonEvent>,
) {
    for _ in start_btn_rdr.read() {
        info!("start button clicked!");
        if let Some(id) = input.gamepad_first() {
            rumble_manager.add_rumble(id, Duration::from_millis(200), GamepadRumbleIntensity::strong_motor(0.4));
        }
    }
    for _ in continue_btn_rdr.read() {
        info!("continue button clicked!");
        if let Some(id) = input.gamepad_first() {
            rumble_manager.add_rumble(id, Duration::from_millis(200), GamepadRumbleIntensity::strong_motor(0.4));
        }
    }
}
