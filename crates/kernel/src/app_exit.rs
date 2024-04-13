use std::sync::{Arc, RwLock};

use bevy_app::AppExit;
use bevy_ecs::{event::{Event, EventReader}, system::NonSendMut};
use logging::info;

use render_gl::window::{FrameInput, OutgoingEvent};
use crate::exit_action_container::ExitActionContainer;


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

// used at a system, setup in EnginePlugin
pub fn process(
    mut frame_input: NonSendMut<FrameInput>,
    mut exit_event_reader: EventReader<AppExit>,
    mut exit_action_event_reader: EventReader<AppExitAction>,
) {
    if ExitActionContainer::is_set() {
        return;
    }
    // read exit action events
    if let Some(first_action) = exit_action_event_reader.read().next() {

        info!("system received exit action event: {:?}", first_action);

        frame_input.outgoing_events.push(OutgoingEvent::Exit);

        // get action string
        let action_string = match first_action {
            AppExitAction::JustExit => "exit".to_string(),
            AppExitAction::GoTo(app_name) => app_name.clone(),
        };

        // store action
        ExitActionContainer::set(action_string);
        return;
    }

    // read exit events
    for _ in exit_event_reader.read() {
        frame_input.outgoing_events.push(OutgoingEvent::Exit);

        // get action string
        let action_string = "exit".to_string();

        // store action
        ExitActionContainer::set(action_string);
        return;
    }
}