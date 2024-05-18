use asset_id::AssetId;

#[derive(Clone)]
pub struct UiContainerState {
    pub ui_handle_opt: Option<AssetId>,
}

impl UiContainerState {
    pub fn new() -> Self {
        Self {
            ui_handle_opt: None,
        }
    }
}
