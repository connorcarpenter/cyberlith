
use spec::AnimatedModel;

use crate::json::AnimatedModelJson;

impl From<&AnimatedModel> for AnimatedModelJson {
    fn from(value: &AnimatedModel) -> Self {
        let mut me = Self::new();
        me.set_model_asset_id(&value.get_model_asset_id());
        for (name, asset_id) in value.get_animations() {
            me.add_animation(name, asset_id);
        }

        me
    }
}

// impl AnimatedModelJson {
//     pub fn write(&self, asset_id: &AssetId) -> Box<[u8]> {
//         let new_meta = AssetMeta::new(asset_id, Self::CURRENT_SCHEMA_VERSION);
//         let asset = Asset::new(new_meta, AssetData::AnimatedModel(self.clone()));
//         serde_json::to_vec_pretty(&asset)
//             .unwrap()
//             .into_boxed_slice()
//     }
// }