
use ui::UiConfig;

use crate::{asset_dependency::AssetDependency, AssetHandle, IconData, TypedAssetId};

pub struct UiConfigData {
    icon_file: AssetDependency<IconData>,
    ui_config: UiConfig,
}

impl Default for UiConfigData {
    fn default() -> Self {
        panic!("");
    }
}

impl UiConfigData {
    pub fn from_ui_config(ui_config: UiConfig) -> Self {
        let icon_asset_id = ui_config.get_text_icon_asset_id();
        let icon_file = AssetDependency::AssetId(*icon_asset_id);

        Self { icon_file, ui_config }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let ui = asset_serde::bits::read_ui_bits(bytes);
        Self::from_ui_config(ui)
    }

    pub fn get_ui_config_ref(&self) -> &UiConfig {
        &self.ui_config
    }

    pub(crate) fn get_ui_config_mut(&mut self) -> &mut UiConfig {
        &mut self.ui_config
    }

    pub(crate) fn load_dependencies(
        &self,
        asset_handle: AssetHandle<Self>,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        let AssetDependency::<IconData>::AssetId(asset_id) = &self.icon_file else {
            panic!("expected path right after load");
        };
        dependencies.push((asset_handle.into(), TypedAssetId::Icon(*asset_id)));
    }

    pub(crate) fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
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
