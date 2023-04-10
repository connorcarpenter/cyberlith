use std::default::Default;

use bevy_ecs::system::Resource;

pub enum WorkspaceType {
    None,
    SkeletonBuilder,
    TextEditor,
}

#[derive(Resource)]
pub struct UiState {
    pub workspace_type: WorkspaceType,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            workspace_type: WorkspaceType::None,
        }
    }
}
