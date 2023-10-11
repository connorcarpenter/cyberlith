use bevy_ecs::{entity::Entity, system::Resource};

use render_egui::egui::Pos2;

use vortex_proto::components::FileExtension;

use crate::app::ui::text_input_modal::TextInputModal;

#[derive(PartialEq)]
pub enum LoggingInState {
    NotLoggingIn,
    LoggingIn,
    LoginFailed,
}

#[derive(Clone, Copy, PartialEq)]
pub enum BindingState {
    NotBinding,
    Binding(FileExtension),
    // file entity
    BindResult(Entity),
}

#[derive(Resource)]
pub struct UiState {
    pub logged_in: bool,
    pub logging_in_state: LoggingInState,
    pub username: String,
    pub password: String,
    pub text_input_modal: TextInputModal,
    pub dragging_side_panel: bool,
    pub canvas_coords: Option<Pos2>,
    pub resized_window: bool,
    pub binding_file: BindingState,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            logged_in: false,
            logging_in_state: LoggingInState::NotLoggingIn,
            username: String::new(),
            password: String::new(),
            text_input_modal: TextInputModal::new(),
            dragging_side_panel: false,
            canvas_coords: None,
            resized_window: false,
            binding_file: BindingState::NotBinding,
        }
    }
}
