use bevy_ecs::system::Resource;

use render_egui::egui::Pos2;

use crate::app::ui::text_input_modal::TextInputModal;

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
    pub text_input_modal: TextInputModal,
    pub dragging_side_panel: bool,
    pub workspace_coords: Option<Pos2>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            logged_in: false,
            logging_in_state: LoggingInState::NotLoggingIn,
            username: String::new(),
            password: String::new(),
            workspace_type: WorkspaceType::SkeletonBuilder,
            text_input_modal: TextInputModal::new(),
            dragging_side_panel: false,
            workspace_coords: None,
        }
    }
}

#[derive(Resource)]
pub struct AxesCamerasVisible(pub(crate) bool);
