use std::{time::Duration, collections::BTreeMap};

use bevy_ecs::{
    event::{Event, EventReader, EventWriter},
    prelude::Commands,
    system::{Res, ResMut},
};

use game_engine::{
    asset::{
        embedded_asset_event, AssetId, AssetMetadataSerde, AssetType, ETag, EmbeddedAssetEvent,
    },
    input::{GamepadRumbleIntensity, Input, RumbleManager},
    render::components::{
        Camera, CameraBundle, ClearOperation, OrthographicProjection, Projection, RenderLayers,
        RenderTarget,
    },
    ui::UiManager,
};

use asset_serde::json::{Asset, AssetData, AssetMeta, UiConfigJson};
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
    mut global: ResMut<Global>,
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
    uis.push(game::global_chat_day_divider::ui_define()); // game global chat day divider
    uis.push(game::global_chat_username_and_message::ui_define()); // game global chat username and message
    uis.push(game::global_chat_message::ui_define()); // game global chat message

    for (ui_name, ui_asset_id, ui_etag, ui) in uis {
        // write JSON and bits files, metadata too
        let ui = write_to_file(&ui_name, &ui_asset_id, &ui_etag, ui);

        // load ui into asset manager
        let ui_handle = ui_manager
            .manual_load_ui_config(&ui_asset_id, UiRuntimeConfig::load_from_builder_config(ui));

        global.ui_handles.push(ui_handle);
    }

    ui_manager.set_target_render_layer(RenderLayers::layer(0));
    ui_manager.enable_ui(&global.ui_handles[0]);

    setup_global_chat_test_case(&mut global, &mut ui_manager);

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

fn setup_global_chat_test_case(
    global: &mut Global,
    ui_manager: &mut UiManager,
) {
    // main menu ui
    let main_menu_ui_handle = global.ui_handles[0];

    // global chat sub-ui
    let global_chat_ui_handle = global.ui_handles[1];

    // setup sub ui
    ui_manager.set_ui_container_contents(&main_menu_ui_handle, "center_container", &global_chat_ui_handle);

    // setup global chat list
    global.list_ui_ext.set_container_ui(ui_manager, &global_chat_ui_handle, "chat_wall");

    // setup chats
    global.global_chats = setup_global_chats();

    global.sync_chat_collections(ui_manager);
}

fn setup_global_chats() -> BTreeMap<u32, (String, u8, u8, u8, u8, String)> {
    let mut users = Vec::new();
    users.push("tom"); users.push("ben"); users.push("andrew"); users.push("joe");
    users.push("jane"); users.push("sarah"); users.push("jim"); users.push("bob");

    let mut messages = Vec::new();
    messages.push("hello"); messages.push("woah"); messages.push("jeesh");
    messages.push("mmkay"); messages.push("huh"); messages.push("what");
    messages.push("ok"); messages.push("sure"); messages.push("nope");
    messages.push("yep"); messages.push("maybe"); messages.push("never");
    messages.push("always"); messages.push("sometimes"); messages.push("often");
    messages.push("rarely"); messages.push("blah"); messages.push("meh");

    let mut global_chats = BTreeMap::<u32, (String, u8, u8, u8, u8, String)>::new();

    let mut current_time = (3, 1, 11, 30);
    let mut current_user_index = 0;

    for _i in 0..32 {
        if random::gen_range_u32(0, 5) < 1 {
            current_user_index = random::gen_range_u32(0, users.len() as u32) as usize;
        }
        let message_index = random::gen_range_u32(0, messages.len() as u32) as usize;
        setup_global_chat(&mut global_chats, &mut current_time, users[current_user_index], messages[message_index]);
    }

    global_chats
}

fn setup_global_chat(
    global_chats: &mut BTreeMap<u32, (String, u8, u8, u8, u8, String)>,
    current_time: &mut (u32, u32, u32, u32),
    username: &str,
    message: &str,
) {
    let (month, day, hour, minute) = current_time;

    global_chats.insert(
        global_chats.len() as u32,
        (username.to_string(), *month as u8, *day as u8, *hour as u8, *minute as u8, message.to_string())
    );

    let add_minutes = random::gen_range_u32(1, 300); // 1 minutes to 1/2 day
    *minute += add_minutes;
    while *minute >= 60 {
        *minute -= 60;
        *hour += 1;
    }
    while *hour >= 24 {
        *hour -= 24;
        *day += 1;
    }
    while *day >= 31 {
        *day -= 31;
        *month += 1;
    }
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
