
use bevy_ecs::event::Event;

#[derive(Event, Debug)]
pub enum AppExitAction {
    JustExit,
    GoTo(String),
}

impl AppExitAction {
    pub fn just_exit() -> Self {
        AppExitAction::JustExit
    }

    pub fn go_to(app_name: &str) -> Self {
        AppExitAction::GoTo(app_name.to_string())
    }
}