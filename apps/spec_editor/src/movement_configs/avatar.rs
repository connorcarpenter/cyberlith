use asset_id::{AssetId, ETag};
use spec::MovementConfig;

#[allow(unused)]
pub fn define() -> (String, AssetId, ETag, MovementConfig) {
    // config
    let self_name = "avatar";
    let self_asset_id_str = "wyte5b"; // AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let self_etag = ETag::gen_random();

    // asset ids ..
    let self_asset_id = AssetId::from_str(&self_asset_id_str).unwrap();

    // Create spec !
    let mut movement_config = MovementConfig::new(2.0);

    (self_name.to_string(), self_asset_id, self_etag, movement_config)
}
