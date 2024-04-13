use std::sync::{Arc, RwLock};

use bevy_app::AppExit;
use bevy_ecs::{event::{Event, EventReader}, system::NonSendMut};

use render_gl::window::{FrameInput, OutgoingEvent};

pub(crate) static mut EXIT_ACTION_CONTAINER: Option<Arc<RwLock<String>>> = None;
pub(crate) struct ExitActionContainer;
impl ExitActionContainer {
    pub fn is_set() -> bool {
        unsafe { EXIT_ACTION_CONTAINER.is_some() }
    }
    pub fn set(action: String) {
        unsafe {
            if EXIT_ACTION_CONTAINER.is_some() {
                panic!("ExitActionContainer already set");
            }
            EXIT_ACTION_CONTAINER = Some(Arc::new(RwLock::new(action)));
        }
    }
    pub fn take() -> String {
        unsafe {
            let output = EXIT_ACTION_CONTAINER
                .as_ref()
                .unwrap()
                .read()
                .unwrap()
                .clone();
            EXIT_ACTION_CONTAINER = None;
            output
        }
    }
}

#[derive(Event)]
pub enum AppExitAction {
    JustExit,
    GoTo(String),
}

impl AppExitAction {
    pub fn just_exit() -> Self {
        AppExitAction::JustExit
    }

    pub fn go_to(app_name: String) -> Self {
        AppExitAction::GoTo(app_name)
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