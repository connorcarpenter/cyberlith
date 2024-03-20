use bevy_ecs::{prelude::Commands, event::{Event, EventReader, EventWriter}, change_detection::ResMut};
use bevy_log::info;

use asset_io::json::{Asset, AssetData, AssetMeta, UiJson};
use game_engine::{
    asset::{AssetMetadataSerde, AssetHandle, AssetManager, AssetType, embedded_asset_event, EmbeddedAssetEvent, UiData, AssetId, ETag},
    render::base::Color,
    ui::{Alignment, Ui},
};

// this is where new UIs should be created

pub fn init_ui() -> (String, AssetId, ETag, Ui) {

    // config
    let ui_name = "main";
    let ui_asset_id_str = "tpp7za"; // AssetId::get_random(); // keep this around to generate new AssetIds if needed!
    let icon_asset_id_str = "34mvvk";
    let ui_etag = ETag::new_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(ui_asset_id_str).unwrap();
    let icon_asset_id = AssetId::from_str(icon_asset_id_str).unwrap();

    // Create UI !
    let mut ui = Ui::new();

    // styles
    let window_style = ui.create_panel_style(|s| {
        s
            //.set_background_color(Color::BLACK)
            .set_background_alpha(0.0)
            .set_padding_px(10.0, 10.0, 10.0, 10.0)
            .set_vertical()
            .set_row_between_px(10.0);
    });
    let container_style = ui.create_panel_style(|s| {
        s.set_background_alpha(0.0)
            .set_size_pc(100., 38.)
            .set_solid_fit()
            .set_aspect_ratio(16., 4.);
    });
    let base_button_style = ui.create_button_style(|s| {
        s.set_background_color(Color::DARK_GRAY)
            .set_hover_color(Color::RED)
            .set_down_color(Color::BLUE)
            .set_self_halign(Alignment::Center)
            .set_size_pc(50.0, 20.0)
            .set_size_max_px(240.0, 90.0)
            .set_solid_fit()
            .set_aspect_ratio(16.0, 4.)
            .set_padding_px(10.0, 10.0, 10.0, 10.0);
    });
    let start_button_style = ui.create_button_style(|s| {
        s.set_margin_right_px(40.0);
    });
    let continue_button_style = ui.create_button_style(|s| {
        s.set_margin_left_px(40.0);
    });

    // nodes
    ui
        .set_text_icon_asset_id(&icon_asset_id)
        .set_text_color(Color::WHITE)
        .root_mut()
        .add_style(window_style)
        .contents(|c| {
            // title container
            c.add_panel().add_style(container_style).contents(|c| {
                c.add_text("c y b e r l i t h");
            });

            // start button
            c.add_button("start_button")
                .add_style(base_button_style)
                .add_style(start_button_style)
                .contents(|c| {
                    c.add_text("start");
                });

            // continue button
            c.add_button("continue_button")
                .add_style(base_button_style)
                .add_style(continue_button_style)
                .contents(|c| {
                    c.add_text("continue");
                });
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui)
}

#[derive(Event)]
pub struct StartButtonEvent;

#[derive(Event)]
pub struct ContinueButtonEvent;

pub fn setup_ui(
    mut commands: Commands,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
    mut asset_manager: ResMut<AssetManager>,
) {
    embedded_asset_events.send(embedded_asset_event!("embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("embedded/34mvvk")); // verdana icon

    // create ui
    let (ui_name, ui_asset_id, ui_etag, ui) = init_ui();

    // finish

    // write JSON and bits files, metadata too
    let ui = write_to_file(&ui_name, &ui_asset_id, &ui_etag, ui);

    // load ui into asset manager
    asset_manager.manual_load_ui(&ui_asset_id, ui);

    // make handle, add handle to entity
    let ui_handle = AssetHandle::<UiData>::new(ui_asset_id);
    let ui_entity = commands.spawn(ui_handle).id();

    asset_manager.register_event::<StartButtonEvent>(ui_entity, ui_handle, "start_button");
    asset_manager.register_event::<ContinueButtonEvent>(ui_entity, ui_handle, "continue_button");
}

pub fn handle_events(
    mut start_btn_rdr: EventReader<StartButtonEvent>,
    mut continue_btn_rdr: EventReader<ContinueButtonEvent>
) {
    for _ in start_btn_rdr.read() {
        info!("start button clicked!");
    }
    for _ in continue_btn_rdr.read() {
        info!("continue button clicked!");
    }
}

fn write_to_file(name: &str, ui_asset_id: &AssetId, ui_etag: &ETag, ui: Ui) -> Ui {
    let ui_asset_id_str = ui_asset_id.to_string();

    // ui -> JSON bytes
    let ui_bytes = {
        let ui_json = UiJson::from_ui(&ui);
        let new_meta = AssetMeta::new(&ui_asset_id, UiJson::CURRENT_SCHEMA_VERSION);
        let asset = Asset::new(new_meta, AssetData::Ui(ui_json));
        let ui_bytes = serde_json::to_vec_pretty(&asset)
            .unwrap();
        info!("json byte count: {:?}", ui_bytes.len());
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
    info!("bits byte count: {:?}", ui_bytes.len());

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