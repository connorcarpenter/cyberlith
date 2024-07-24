use asset_id::AssetId;
use spec::AnimatedModel;

use crate::json::AnimatedModelJson;

impl Into<AnimatedModel> for AnimatedModelJson {
    fn into(self) -> AnimatedModel {
        let mut me = AnimatedModel::new(self.get_model_asset_id());
        for (name, asset_id) in self.get_animations() {
            me.add_animation(name, AssetId::from_str(asset_id).unwrap());
        }

        me
    }
}

// impl AnimatedModelJson {
//     pub fn read(bytes: &[u8]) -> Result<(AssetMeta, Self), AssetIoError> {
//         let (meta, data) = Asset::read(bytes)?.deconstruct();
//         let AssetData::AnimatedModel(data) = data else {
//             return Err(AssetIoError::Message("Invalid Asset Type".to_string()));
//         };
//         return Ok((meta, data));
//     }
// }
