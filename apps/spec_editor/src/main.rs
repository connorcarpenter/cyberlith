use asset_id::{AssetId, AssetType, ETag};
use asset_serde::{bits::AssetMetadataSerde, json::{Asset, AssetData, AssetMeta, UiConfigJson}};
use ui_builder::UiConfig;

mod main_menu;

fn main() {
    setup();
}

pub fn setup() {
    load_ui(main_menu::ui_define()); // game main menu
}

pub(crate) fn load_ui(
    ui_define: (String, AssetId, ETag, UiConfig),
) {
    let (ui_name, ui_asset_id, ui_etag, ui) = ui_define;

    write_to_file(&ui_name, &ui_asset_id, &ui_etag, ui);
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

    // delete bit-packed files
    std::fs::remove_file(format!("output/{}", ui_asset_id_str)).unwrap();
    std::fs::remove_file(format!("output/{}.meta", ui_asset_id_str)).unwrap();

    ui
}