
use bevy_ecs::{system::ResMut, event::EventReader};

use game_engine::{ui::UiManager, input::{InputEvent, Key}};

use crate::app::global::Global;

pub fn scroll_events(
    mut global: ResMut<Global>,
    mut ui_manager: ResMut<UiManager>,
    mut input_events: EventReader<InputEvent>,
) {
    for event in input_events.read() {
        match event {
            InputEvent::KeyPressed(Key::I, _) => {
                global.scroll_up(&mut ui_manager);
            }
            InputEvent::KeyPressed(Key::K, _) => {
                global.scroll_down(&mut ui_manager);
            }
            _ => {}
        }
    }
}