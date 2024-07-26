use bevy_ecs::{event::EventWriter, system::Resource};

use crate::ui::events::ResyncMainMenuUiEvent;

#[derive(Resource)]
pub struct MatchManager {
    in_match: bool,
}

impl Default for MatchManager {
    fn default() -> Self {
        Self { in_match: false }
    }
}

impl MatchManager {
    pub fn in_match(&self) -> bool {
        self.in_match
    }

    pub fn start_match(
        &mut self,
        resync_main_menu_ui_events: &mut EventWriter<ResyncMainMenuUiEvent>,
    ) {
        self.in_match = true;

        resync_main_menu_ui_events.send(ResyncMainMenuUiEvent);
    }

    pub fn leave_match(&mut self) {
        self.in_match = false;
    }
}
