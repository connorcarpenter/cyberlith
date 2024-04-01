use std::time::Duration;

use bevy_ecs::{
    event::{Event, EventReader, EventWriter},
    prelude::Commands,
    system::{Res, Query, ResMut},
};
use bevy_log::{info, warn};

use asset_serde::json::{Asset, AssetData, AssetMeta, UiConfigJson};
use game_engine::{
    asset::{
        embedded_asset_event, AssetHandle, AssetId, AssetManager, AssetMetadataSerde, AssetType,
        ETag, EmbeddedAssetEvent,
    },
    render::{base::Color, components::{AmbientLight, CameraBundle, ClearOperation, OrthographicProjection, Projection, RenderTarget, RenderLayer, Camera}},
    ui::{UiRuntime, UiManager, UiInputConverter},
    input::{Input, InputEvent, GamepadRumbleIntensity, RumbleManager},
};
use ui_builder::UiConfig;

use crate::app::{ui_backups::*, global::Global};

fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // start
    //return start::ui_define();

    // login
    // return login::ui_define();

    // register
    return register::ui_define();
}

#[derive(Event, Default)]
pub struct SubmitButtonEvent;

// this is run as a system at startup
pub fn setup(
    mut commands: Commands,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
    mut ui_manager: ResMut<UiManager>,
) {
    // ui setup
    embedded_asset_events.send(embedded_asset_event!("embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("embedded/34mvvk")); // verdana icon

    // create ui
    let (ui_name, ui_asset_id, ui_etag, ui) = ui_define();

    // write JSON and bits files, metadata too
    let ui = write_to_file(&ui_name, &ui_asset_id, &ui_etag, ui);

    // load ui into asset manager
    ui_manager.manual_load_ui_config(&ui_asset_id, ui);

    // make handle, add handle to entity
    let ui_handle = AssetHandle::<UiRuntime>::new(ui_asset_id);
    let ui_entity = commands.spawn(ui_handle).id();

    ui_manager.register_ui_event::<SubmitButtonEvent>(&ui_handle, "login_button");

    // scene setup now
    // ambient light
    commands.spawn(AmbientLight::new(1.0, Color::WHITE));

    // camera
    let ui_camera_entity = commands
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
        .id();

    commands.insert_resource(Global::new(ui_camera_entity, ui_entity));
}

pub fn ui_update(
    global: Res<Global>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut input_events: EventReader<InputEvent>,
    // Cameras
    cameras_q: Query<(&Camera, Option<&RenderLayer>)>,
    // UIs
    uis_q: Query<(&AssetHandle<UiRuntime>, Option<&RenderLayer>)>,
) {
    let Ok((ui_handle, ui_render_layer_opt)) = uis_q.get(global.active_ui_entity) else {
        warn!("no active ui entity!");
        return;
    };

    // find camera, update viewport
    let Ok((camera, cam_render_layer_opt)) = cameras_q.get(global.ui_camera_entity) else {
        warn!("no ui camera!");
        return;
    };
    if cam_render_layer_opt == ui_render_layer_opt {
        ui_manager.update_ui_viewport(&asset_manager, camera, ui_handle);
    }

    // update with inputs
    let Some((mouse_position, ui_input_events)) = UiInputConverter::convert(&mut input_events) else {
        return;
    };
    ui_manager.update_ui_input(&asset_manager, ui_handle, mouse_position, ui_input_events);
}

pub fn ui_handle_events(
    input: Res<Input>,
    mut rumble_manager: ResMut<RumbleManager>,
    mut login_btn_rdr: EventReader<SubmitButtonEvent>,
) {
    for _ in login_btn_rdr.read() {
        info!("login button clicked!");
        if let Some(id) = input.gamepad_first() {
            rumble_manager.add_rumble(id, Duration::from_millis(200), GamepadRumbleIntensity::weak_motor(0.4));
        }
    }
}

fn write_to_file(name: &str, ui_asset_id: &AssetId, ui_etag: &ETag, ui: UiConfig) -> UiConfig {
    let ui_asset_id_str = ui_asset_id.to_string();

    // ui -> JSON bytes
    let ui_bytes = {
        let ui_json = UiConfigJson::from(&ui);
        let new_meta = AssetMeta::new(&ui_asset_id, UiConfigJson::CURRENT_SCHEMA_VERSION);
        let asset = Asset::new(new_meta, AssetData::Ui(ui_json));
        let ui_bytes = serde_json::to_vec_pretty(&asset).unwrap();
        // info!("json byte count: {:?}", ui_bytes.len());
        ui_bytes
    };

    // write JSON bytes to file
    std::fs::write(format!("output/{}.ui.json", name), &ui_bytes).unwrap();

    // JSON bytes -> ui
    let ui = {
        let asset: Asset = serde_json::from_slice(&ui_bytes).unwrap();
        let (_, data) = asset.deconstruct();
        let AssetData::Ui(ui_json) = data else {
            panic!("expected UiData");
        };
        ui_json.into()
    };

    // ui -> bit-packed bytes
    let ui_bytes = asset_serde::bits::write_ui_bits(&ui);
    // info!("bits byte count: {:?}", ui_bytes.len());

    // write bit-packed data to file
    std::fs::write(format!("output/{}", ui_asset_id_str), &ui_bytes).unwrap();

    // write metadata to file
    {
        let ui_metadata = AssetMetadataSerde::new(*ui_etag, AssetType::Ui);
        let metadata_bytes = ui_metadata.to_bytes();
        std::fs::write(format!("output/{}.meta", ui_asset_id_str), &metadata_bytes).unwrap();
    }

    // bit-packed bytes -> ui
    let ui = asset_serde::bits::read_ui_bits(&ui_bytes);
    ui
}
