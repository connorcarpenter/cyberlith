use std::default::Default;

use bevy_ecs::system::Resource;

#[derive(PartialEq)]
pub enum WorkspaceType {
    None,
    SkeletonBuilder,
    TextEditor,
}

#[derive(PartialEq)]
pub enum LoggingInState {
    NotLoggingIn,
    LoggingIn,
    LoginFailed,
}

#[derive(Resource)]
pub struct UiState {
    pub logged_in: bool,
    pub logging_in_state: LoggingInState,
    pub username: String,
    pub password: String,
    pub workspace_type: WorkspaceType,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            logged_in: false,
            logging_in_state: LoggingInState::NotLoggingIn,
            username: String::new(),
            password: String::new(),
            workspace_type: WorkspaceType::SkeletonBuilder,
        }
    }
}

#[derive(Resource)]
pub struct AxesCamerasVisible(pub(crate) bool);
