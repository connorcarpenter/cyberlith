
use ui_types::UiConfig;

use crate::{asset_dependency::AssetDependency, AssetHandle, IconData, TypedAssetId};

pub struct UiConfigData {
    icon_file: AssetDependency<IconData>,
}

impl Default for UiConfigData {
    fn default() -> Self {
        panic!("");
    }
}

impl UiConfigData {
    pub fn from_ui_config(ui_config: UiConfig) -> (Self, UiConfig) {
        let icon_asset_id = ui_config.get_text_icon_asset_id();
        let icon_file = AssetDependency::AssetId(*icon_asset_id);

        (Self { icon_file }, ui_config)
    }

    pub fn from_bytes(bytes: &[u8]) -> (Self, UiConfig) {
        let ui = asset_serde::bits::read_ui_bits(bytes);
        Self::from_ui_config(ui)
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
