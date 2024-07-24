use crate::TypedAssetId;

pub struct UiDependencies {}

impl Default for UiDependencies {
    fn default() -> Self {
        panic!("");
    }
}

impl UiDependencies {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_dependencies(
        &self,
        _asset_handle: TypedAssetId,
        _dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
    }

    pub fn finish_dependency(&mut self, _dependency_typed_id: TypedAssetId) {
        panic!("unexpected icon id");
    }
}
