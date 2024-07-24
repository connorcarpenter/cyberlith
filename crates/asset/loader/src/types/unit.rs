
use asset_serde::bits::UnitBits;

use crate::{AnimatedModelData, asset_dependency::AssetDependency, AssetHandle, MovementConfigData, TypedAssetId};

pub struct UnitData {
    animated_model_file: AssetDependency<AnimatedModelData>,
    movement_config_file: AssetDependency<MovementConfigData>,
}

impl Default for UnitData {
    fn default() -> Self {
        panic!("");
    }
}

impl From<&[u8]> for UnitData {
    fn from(bytes: &[u8]) -> Self {
        // info!("--- reading unit ---");

        let base = UnitBits::from_bytes(bytes).expect("unable to parse file");

        let animated_model_asset_id = base.get_animated_model_asset_id();
        let movement_config_asset_id = base.get_movement_config_asset_id();

        // info!("--- done reading unit ---");

        Self {
            animated_model_file: AssetDependency::AssetId(animated_model_asset_id),
            movement_config_file: AssetDependency::AssetId(movement_config_asset_id),
        }
    }
}

impl UnitData {
    pub fn get_animated_model_file_handle(&self) -> Option<&AssetHandle<AnimatedModelData>> {
        if let AssetDependency::<AnimatedModelData>::AssetHandle(handle) = &self.animated_model_file {
            Some(handle)
        } else {
            None
        }
    }

    pub fn get_movement_config_file_handle(&self) -> Option<&AssetHandle<MovementConfigData>> {
        if let AssetDependency::<MovementConfigData>::AssetHandle(handle) = &self.movement_config_file {
            Some(handle)
        } else {
            None
        }
    }

    pub(crate) fn load_dependencies(
        &self,
        handle: AssetHandle<Self>,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        let AssetDependency::<AnimatedModelData>::AssetId(asset_id) = &self.animated_model_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), TypedAssetId::AnimatedModel(asset_id.clone())));

        let AssetDependency::<MovementConfigData>::AssetId(asset_id) = &self.movement_config_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), TypedAssetId::MovementConfig(asset_id.clone())));
    }

    pub(crate) fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        match dependency_typed_id {
            TypedAssetId::AnimatedModel(asset_id) => {
                let handle = AssetHandle::<AnimatedModelData>::new(asset_id);
                self.animated_model_file.load_asset_handle(handle);
            }
            TypedAssetId::MovementConfig(asset_id) => {
                let handle = AssetHandle::<MovementConfigData>::new(asset_id);
                self.movement_config_file.load_asset_handle(handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }
}
