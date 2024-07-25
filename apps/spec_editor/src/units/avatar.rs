use asset_id::{AssetId, ETag};
use spec::Unit;

use crate::catalog::AssetCatalog;

#[allow(unused)]
pub fn define() -> (String, AssetId, ETag, Unit) {
    // config
    let self_name = "avatar";
    let self_asset_id_str = "h1g2dt"; // AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let self_etag = ETag::gen_random();

    // asset ids ..
    let self_asset_id = AssetId::from_str(&self_asset_id_str).unwrap();

    let animated_model_asset_id = AssetCatalog::avatar_animated_model();
    let movement_config_asset_id = AssetCatalog::avatar_movement_config();

    // Create spec !
    let mut unit = Unit::new(animated_model_asset_id, movement_config_asset_id);

    (self_name.to_string(), self_asset_id, self_etag, unit)
}
