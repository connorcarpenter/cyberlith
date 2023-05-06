use std::default::Default;

use bevy_ecs::system::Resource;
use egui_modal::Modal;
use render_egui::egui;
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
        }
    }
}

#[derive(Resource)]
pub struct AxesCamerasVisible(pub(crate) bool);
