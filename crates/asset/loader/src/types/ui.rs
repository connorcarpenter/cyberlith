use asset_id::AssetId;
use ui_types::UiConfig;

use crate::{asset_dependency::AssetDependency, AssetHandle, IconData, TypedAssetId};

pub struct UiDependencies {
    icon_file: AssetDependency<IconData>,
}

impl Default for UiDependencies {
    fn default() -> Self {
        panic!("");
    }
}

impl UiDependencies {

    pub fn new(icon_asset_id: &AssetId) -> Self {
        let icon_file = AssetDependency::AssetId(*icon_asset_id);

        Self {
            icon_file,
        }
    }

    pub fn load_config_from_bytes(bytes: &[u8]) -> UiConfig {
        asset_serde::bits::read_ui_bits(bytes)
    }

    pub fn load_dependencies(
        &self,
        asset_handle: TypedAssetId,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        let AssetDependency::<IconData>::AssetId(asset_id) = &self.icon_file else {
            panic!("expected path right after load");
        };
        dependencies.push((asset_handle, TypedAssetId::Icon(*asset_id)));
    }

    pub fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        match dependency_typed_id {
            TypedAssetId::Icon(id) => {
                let handle = AssetHandle::<IconData>::new(id);
                self.icon_file.load_asset_handle(handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }

    pub fn get_icon_handle(&self) -> AssetHandle<IconData> {
        if let AssetDependency::<IconData>::AssetHandle(handle) = &self.icon_file {
            *handle
        } else {
            panic!("expected icon handle");
        }
    }
}
