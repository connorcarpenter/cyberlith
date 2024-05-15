use std::time::Duration;

use bevy_ecs::{
    event::{Event, EventReader, EventWriter},
    prelude::Commands,
    system::{Res, ResMut},
};
use logging::info;

use asset_serde::json::{Asset, AssetData, AssetMeta, UiConfigJson};
use game_engine::{
    asset::{
        embedded_asset_event, AssetId, AssetMetadataSerde, AssetType, ETag, EmbeddedAssetEvent,
    },
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    render::{
        components::{
            RenderLayers, Camera, CameraBundle, ClearOperation, OrthographicProjection, Projection,
            RenderTarget,
        },
    },
    ui::UiManager,
};
use ui_builder::UiConfig;
use ui_runner_config::UiRuntimeConfig;

use crate::app::{global::Global, ui_backups::*};

fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // start
    // return start::ui_define();

    // login
    // return login::ui_define();

    // register
    // return register::ui_define();

    // register_finish
    // return register_finish::ui_define();

    // forgot username
    // return forgot_username::ui_define();

    // forgot username finish
    return forgot_username_finish::ui_define();

    // forgot password
    // return forgot_password::ui_define();

    // forgot password finish
    // return forgot_password_finish::ui_define();
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
    embedded_asset_events.send(embedded_asset_event!("embedded/qbgz5j")); // password eye icon

    // create ui
    let (ui_name, ui_asset_id, ui_etag, ui) = ui_define();

    // write JSON and bits files, metadata too
    let ui = write_to_file(&ui_name, &ui_asset_id, &ui_etag, ui);

    // load ui into asset manager
    let ui_handle = ui_manager
        .manual_load_ui_config(&ui_asset_id, UiRuntimeConfig::load_from_builder_config(ui));

    ui_manager.set_target_render_layer(RenderLayers::layer(0));
    ui_manager.enable_ui(&ui_handle);
    // ui_manager.register_ui_event::<SubmitButtonEvent>(&ui_handle, "login_button");

    // scene setup now
    // ambient light
    // commands.spawn(AmbientLight::new(1.0, Color::WHITE)).insert(RenderLayers::layer(0));
    //
    // // camera
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

    commands.insert_resource(Global::new(scene_camera_entity));
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
    let Ok(ui) = asset_serde::bits::read_ui_bits(&ui_bytes) else {
        panic!("failed to read ui bits for asset_id: {:?}", ui_asset_id);
    };
    ui
}
