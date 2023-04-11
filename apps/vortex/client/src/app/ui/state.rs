use std::default::Default;

use bevy_ecs::system::Resource;

#[derive(PartialEq)]
pub enum WorkspaceType {
    None,
    SkeletonBuilder,
    TextEditor,
}

#[derive(Resource)]
pub struct UiState {
    pub logged_in: bool,
    pub username: String,
    pub password: String,
    pub workspace_type: WorkspaceType,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            logged_in: false,
            username: String::new(),
            password: String::new(),
            workspace_type: WorkspaceType::SkeletonBuilder,
        }
    }
}

#[derive(Resource)]
pub struct AxesCamerasVisible(pub(crate) bool);
