use asset_id::{AssetId, AssetType, ETag};
use asset_serde::{
    bits::{AnimatedModelBits, AssetMetadataSerde},
    json::{AnimatedModelJson, Asset, AssetData, AssetMeta},
};
use spec::AnimatedModel;

pub(crate) fn write_to_file(definition: (String, AssetId, ETag, AnimatedModel)) -> AnimatedModel {
    let (name, spec_asset_id, spec_etag, spec) = definition;

    let spec_asset_id_str = spec_asset_id.to_string();

    // spec -> JSON bytes
    let spec_bytes = {
        let spec_json = AnimatedModelJson::from(&spec);
        let new_meta = AssetMeta::new(&spec_asset_id, AnimatedModelJson::CURRENT_SCHEMA_VERSION);
        let asset = Asset::new(new_meta, AssetData::AnimatedModel(spec_json));
        let spec_bytes = serde_json::to_vec_pretty(&asset).unwrap();
        // info!("json byte count: {:?}", spec_bytes.len());
        spec_bytes
    };

    // write JSON bytes to file
    std::fs::write(format!("output/{}.animated_model.json", name), &spec_bytes).unwrap();

    // JSON bytes -> spec
    let spec: AnimatedModel = {
        let asset: Asset = serde_json::from_slice(&spec_bytes).unwrap();
        let (_, data) = asset.deconstruct();
        let AssetData::AnimatedModel(spec_json) = data else {
            panic!("expected SpecData");
        };
        spec_json.into()
    };

    // spec -> bit-packed bytes
    let spec_bytes: Vec<u8> = {
        let spec_bits: AnimatedModelBits = (&spec).into();
        spec_bits.into()
    };
    // info!("bits byte count: {:?}", spec_bytes.len());

    // write bit-packed data to file
    std::fs::write(format!("output/{}", spec_asset_id_str), &spec_bytes).unwrap();

    // write metadata to file
    {
        let metadata = AssetMetadataSerde::new(spec_etag, AssetType::AnimatedModel);
        let metadata_bytes = metadata.to_bytes();
        std::fs::write(
            format!("output/{}.meta", spec_asset_id_str),
            &metadata_bytes,
        )
        .unwrap();
    }

    // bit-packed bytes -> spec
    let animated_model: AnimatedModel = {
        let spec_bits = AnimatedModelBits::from_bytes(&spec_bytes).unwrap();
        AnimatedModelBits::into(spec_bits)
    };

    // delete bit-packed files
    std::fs::remove_file(format!("output/{}", spec_asset_id_str)).unwrap();
    std::fs::remove_file(format!("output/{}.meta", spec_asset_id_str)).unwrap();

    animated_model
}
