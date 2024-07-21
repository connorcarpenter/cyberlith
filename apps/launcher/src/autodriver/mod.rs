use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{event::{EventWriter}, system::{Res, ResMut, Resource}};

use game_engine::{logging::{info, warn}, ui::UiManager};

use crate::{resources::{Global, LoginButtonClickedEvent, SubmitButtonClickedEvent}, ui::UiKey};

pub(crate) struct AutodriverPlugin;

impl Plugin for AutodriverPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AutodriverState>()
            .add_systems(Startup, startup)
            .add_systems(Update, update);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LoginState {
    AtStart,
    LoggedOut,
}

#[derive(Resource)]
struct AutodriverState {
    login_state: LoginState,
    current_username_id: usize,
    usernames: Vec<String>,
}

impl Default for AutodriverState {
    fn default() -> Self {
        Self {
            login_state: LoginState::AtStart,
            current_username_id: 0,
            usernames: vec![
                "connor".to_string(),
                "brendon".to_string(),
                "marina".to_string(),
                "johnny".to_string(),
                "danny".to_string(),
            ],
        }
    }
}

fn startup(
    _state: Res<AutodriverState>,
) {
    info!("Launcher::Autodriver::startup()");
}

fn update(
    mut state: ResMut<AutodriverState>,
    mut ui_manager: ResMut<UiManager>,
    global: Res<Global>,
    mut login_btn_wrtr: EventWriter<LoginButtonClickedEvent>,
    mut submit_btn_wrtr: EventWriter<SubmitButtonClickedEvent>,
) {
    match state.login_state {
        LoginState::AtStart => {
            info!("Launcher::Autodriver::update()::AtStart");

            if let Some(current_ui_handle) = ui_manager.active_ui() {
                match global.get_ui_key(&current_ui_handle) {
                    UiKey::Start => {
                        login_btn_wrtr.send(LoginButtonClickedEvent);
                    }
                    UiKey::Login => {
                        state.login_state = LoginState::LoggedOut;
                    }
                    _ => {
                        warn!("unexpected ui key");
                    }
                }
            }
        }
        LoginState::LoggedOut => {

            let login_ui_handle = global.get_ui_handle(UiKey::Login);
            let error_text = ui_manager.get_text(&login_ui_handle, "error_output_text");
            let has_error_text: bool = {
                if let Some(error_text) = error_text { !error_text.is_empty() } else {
                    false
                }
            };
            let spinner_visibility = ui_manager.get_node_visible(&login_ui_handle, "spinner");

            if spinner_visibility {
                // waiting ...
            } else {
                if has_error_text {

                    // move onto next username
                    state.current_username_id += 1;
                    ui_manager.set_text(&login_ui_handle, "error_output_text", "");

                } else {

                    // no error text, go for it
                    let current_username = state.usernames[state.current_username_id].clone();

                    info!("Autodriver: trying username: {}", current_username);

                    ui_manager
                        .set_text(&login_ui_handle, "username_textbox", &current_username);
                    ui_manager
                        .set_text(&login_ui_handle, "password_textbox", "2simple4u");
                    submit_btn_wrtr.send(SubmitButtonClickedEvent);
                }
            }
        }
    }
}