use asset_id::{AssetId, AssetType, ETag};
use asset_serde::{
    bits::{AssetMetadataSerde, UnitBits},
    json::{Asset, AssetData, AssetMeta, UnitJson},
};
use spec::Unit;

pub(crate) fn write_to_file(definition: (String, AssetId, ETag, Unit)) -> Unit {
    let (name, spec_asset_id, spec_etag, spec) = definition;

    let spec_asset_id_str = spec_asset_id.to_string();

    // spec -> JSON bytes
    let spec_bytes = {
        let spec_json = UnitJson::from(&spec);
        let new_meta = AssetMeta::new(&spec_asset_id, UnitJson::CURRENT_SCHEMA_VERSION);
        let asset = Asset::new(new_meta, AssetData::Unit(spec_json));
        let spec_bytes = serde_json::to_vec_pretty(&asset).unwrap();
        // info!("json byte count: {:?}", spec_bytes.len());
        spec_bytes
    };

    // write JSON bytes to file
    std::fs::write(format!("output/{}.unit.json", name), &spec_bytes).unwrap();

    // JSON bytes -> spec
    let spec: Unit = {
        let asset: Asset = serde_json::from_slice(&spec_bytes).unwrap();
        let (_, data) = asset.deconstruct();
        let AssetData::Unit(spec_json) = data else {
            panic!("expected SpecData");
        };
        spec_json.into()
    };

    // spec -> bit-packed bytes
    let spec_bytes: Vec<u8> = {
        let spec_bits: UnitBits = (&spec).into();
        spec_bits.into()
    };
    // info!("bits byte count: {:?}", spec_bytes.len());

    // write bit-packed data to file
    std::fs::write(format!("output/{}", spec_asset_id_str), &spec_bytes).unwrap();

    // write metadata to file
    {
        let metadata = AssetMetadataSerde::new(spec_etag, AssetType::Unit);
        let metadata_bytes = metadata.to_bytes();
        std::fs::write(
            format!("output/{}.meta", spec_asset_id_str),
            &metadata_bytes,
        )
        .unwrap();
    }

    // bit-packed bytes -> spec
    let unit: Unit = {
        let spec_bits = UnitBits::from_bytes(&spec_bytes).unwrap();
        UnitBits::into(spec_bits)
    };

    // delete bit-packed files
    std::fs::remove_file(format!("output/{}", spec_asset_id_str)).unwrap();
    std::fs::remove_file(format!("output/{}.meta", spec_asset_id_str)).unwrap();

    unit
}
