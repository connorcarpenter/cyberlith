use std::{time::Duration, collections::BTreeMap};

use bevy_ecs::{
    event::{Event, EventReader, EventWriter},
    prelude::Commands,
    system::{Res, ResMut},
};

use asset_serde::json::{Asset, AssetData, AssetMeta, UiConfigJson};
use game_engine::{
    asset::{
        embedded_asset_event, AssetId, AssetMetadataSerde, AssetType, ETag, EmbeddedAssetEvent,
    },
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    render::components::{
        Camera, CameraBundle, ClearOperation, OrthographicProjection, Projection, RenderLayers,
        RenderTarget,
    },
    ui::{UiManager, UiHandle, extensions::ListUiExt},
};
use logging::info;
use ui_builder::UiConfig;
use ui_runner_config::UiRuntimeConfig;

use crate::app::{global::Global, uis::*};

#[derive(Event, Default)]
pub struct SubmitButtonEvent;

// this is run as a system at startup
pub fn setup(
    mut commands: Commands,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
    mut ui_manager: ResMut<UiManager>,
) {
    let mut uis = Vec::new();

    // uis.push(launcher::start::ui_define()); // start
    // uis.push(launcher::login::ui_define()); // login
    // uis.push(launcher::register::ui_define()); // register
    // uis.push(launcher::register_finish::ui_define()); // register_finish
    // uis.push(launcher::forgot_username::ui_define()); // forgot username
    // uis.push(launcher::forgot_username_finish::ui_define()); // forgot username finish
    // uis.push(launcher::forgot_password::ui_define()); // forgot password
    // uis.push(launcher::forgot_password_finish::ui_define()); // forgot password finish
    // uis.push(launcher::reset_password::ui_define()); // reset password

    uis.push(game::main_menu::ui_define()); // game main menu
    // uis.push(game::host_match::ui_define()); // game host match
    uis.push(game::global_chat::ui_define()); // game global chat
    uis.push(game::global_chat_list_item::ui_define()); // game global chat list item

    let mut ui_handles = Vec::new();
    for (ui_name, ui_asset_id, ui_etag, ui) in uis {
        // write JSON and bits files, metadata too
        let ui = write_to_file(&ui_name, &ui_asset_id, &ui_etag, ui);

        // load ui into asset manager
        let ui_handle = ui_manager
            .manual_load_ui_config(&ui_asset_id, UiRuntimeConfig::load_from_builder_config(ui));

        ui_handles.push(ui_handle);
    }

    ui_manager.set_target_render_layer(RenderLayers::layer(0));
    ui_manager.enable_ui(&ui_handles[0]);

    setup_global_chat_test_case(&mut ui_manager, &ui_handles);

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

    // ui setup
    embedded_asset_events.send(embedded_asset_event!("embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("embedded/34mvvk")); // verdana icon
    embedded_asset_events.send(embedded_asset_event!("embedded/qbgz5j")); // password eye icon

    // font & password eye icon setup
    ui_manager.set_text_icon_handle(AssetId::from_str("34mvvk").unwrap());
    ui_manager.set_eye_icon_handle(AssetId::from_str("qbgz5j").unwrap());
}

fn setup_global_chat_test_case(ui_manager: &mut UiManager, ui_handles: &Vec<UiHandle>) {
    // main menu ui
    let main_menu_ui_handle = ui_handles[0];

    // global chat sub-ui
    let global_chat_ui_handle = ui_handles[1];

    // global chat list item ui
    let global_chat_list_item_ui_handle = ui_handles[2];

    // setup sub ui
    ui_manager.set_ui_container_contents(&main_menu_ui_handle, "center_container", &global_chat_ui_handle);

    // setup global chat list
    let mut list_ui_ext = ListUiExt::new();
    list_ui_ext.set_container_ui(ui_manager, &global_chat_ui_handle, "chat_wall");

    // setup collection
    let mut global_chats = BTreeMap::<u32, String>::new();
    global_chats.insert(1, "hello world".to_string());
    global_chats.insert(2, "this is a test".to_string());
    global_chats.insert(3, "this is a test also".to_string());
    global_chats.insert(4, "okay".to_string());
    global_chats.insert(5, "goodbye".to_string());

    // setup collection
    list_ui_ext.sync_with_collection(
        ui_manager,
        &global_chats,
        |item_ctx, _message_id, message_text| {

            item_ctx.add_copied_node(&global_chat_list_item_ui_handle);

            let item_node_id = item_ctx.get_node_id_by_str("message").unwrap();
            item_ctx.set_text(&item_node_id, message_text);
        },
    );
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
