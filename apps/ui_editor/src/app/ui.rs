use std::time::Duration;

use bevy_ecs::{
    event::{Event, EventReader, EventWriter},
    prelude::Commands,
    system::{Res, Query, ResMut},
};
use bevy_log::info;

use asset_io::json::{Asset, AssetData, AssetMeta, UiJson};
use game_engine::{
    asset::{
        embedded_asset_event, AssetHandle, AssetId, AssetManager, AssetMetadataSerde, AssetType,
        ETag, EmbeddedAssetEvent, UiData,
    },
    render::{components::{RenderLayer, Camera}},
    ui::{Ui, UiInputConverter},
    input::{Input, InputEvent, GamepadRumbleIntensity, RumbleManager},
};

use crate::app::ui_backups::*;

fn ui_define() -> (String, AssetId, ETag, Ui) {
    // start
    //return start::ui_define();

    // login
    // return login::ui_define();

    // register
    return register::ui_define();

    // temp
    // return temp::ui_define();
}

#[derive(Event, Default)]
pub struct SubmitButtonEvent;

// this is run as a system at startup
pub fn ui_setup(
    mut commands: Commands,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
    mut asset_manager: ResMut<AssetManager>,
) {
    embedded_asset_events.send(embedded_asset_event!("embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("embedded/34mvvk")); // verdana icon

    // create ui
    let (ui_name, ui_asset_id, ui_etag, ui) = ui_define();

    // finish

    // write JSON and bits files, metadata too
    let ui = write_to_file(&ui_name, &ui_asset_id, &ui_etag, ui);

    // load ui into asset manager
    asset_manager.manual_load_ui(&ui_asset_id, ui);

    // make handle, add handle to entity
    let ui_handle = AssetHandle::<UiData>::new(ui_asset_id);
    let _ui_entity = commands.spawn(ui_handle).id();

    //asset_manager.register_ui_event::<SubmitButtonEvent>(&ui_handle, "login_button");
}

pub fn ui_update(
    mut asset_manager: ResMut<AssetManager>,
    mut input_events: EventReader<InputEvent>,
    // Cameras
    cameras_q: Query<(&Camera, Option<&RenderLayer>)>,
    // UIs
    uis_q: Query<(&AssetHandle<UiData>, Option<&RenderLayer>)>,
) {
    let ui_input = UiInputConverter::convert(&mut input_events);

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
    mut login_btn_rdr: EventReader<SubmitButtonEvent>,
) {
    for _ in login_btn_rdr.read() {
        info!("login button clicked!");
        if let Some(id) = input.gamepad_first() {
            rumble_manager.add_rumble(id, Duration::from_millis(200), GamepadRumbleIntensity::weak_motor(0.4));
        }
    }
}

fn write_to_file(name: &str, ui_asset_id: &AssetId, ui_etag: &ETag, ui: Ui) -> Ui {
    let ui_asset_id_str = ui_asset_id.to_string();

    // ui -> JSON bytes
    let ui_bytes = {
        let ui_json = UiJson::from_ui(&ui);
        let new_meta = AssetMeta::new(&ui_asset_id, UiJson::CURRENT_SCHEMA_VERSION);
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
        ui_json.to_ui()
    };

    // ui -> bit-packed bytes
    let ui_bytes = asset_io::bits::write_ui_bits(&ui);
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
    let ui = asset_io::bits::read_ui_bits(&ui_bytes);
    ui
}
