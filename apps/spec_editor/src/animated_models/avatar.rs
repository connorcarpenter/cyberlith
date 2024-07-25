use asset_id::{AssetId, ETag};
use spec::AnimatedModel;

use crate::catalog::AssetCatalog;

#[allow(unused)]
pub fn define() -> (String, AssetId, ETag, AnimatedModel) {
    // config
    let self_name = "avatar";
    let self_asset_id_str = "2xeqfr"; //AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let self_etag = ETag::gen_random();

    // asset ids ..
    let animated_model_asset_id = AssetId::from_str(&self_asset_id_str).unwrap();

    let model_asset_id = AssetCatalog::avatar_model();
    let idle_animation_asset_id = AssetCatalog::avatar_idle_animation();
    let walk_animation_asset_id = AssetCatalog::avatar_walk_animation();

    // Create spec !
    let mut animated_model = AnimatedModel::new(model_asset_id);
    animated_model.add_animation("idle", idle_animation_asset_id);
    animated_model.add_animation("walk", walk_animation_asset_id);

    (self_name.to_string(), animated_model_asset_id, self_etag, animated_model)
}
