// use crate::{
//     error::AssetIoError,
//     json::{Asset, AssetData, AssetMeta, UnitJson},
// };
//
// impl UnitJson {
//     pub fn read(bytes: &[u8]) -> Result<(AssetMeta, Self), AssetIoError> {
//         let (meta, data) = Asset::read(bytes)?.deconstruct();
//         let AssetData::Unit(data) = data else {
//             return Err(AssetIoError::Message("Invalid Asset Type".to_string()));
//         };
//         return Ok((meta, data));
//     }
// }
