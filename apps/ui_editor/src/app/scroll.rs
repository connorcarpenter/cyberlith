
use bevy_ecs::{system::ResMut, event::EventReader};

use game_engine::input::{InputEvent, Key};

use crate::app::global::Global;

pub fn scroll_events(
    mut global: ResMut<Global>,
    mut input_events: EventReader<InputEvent>,
) {
    for event in input_events.read() {
        match event {
            InputEvent::KeyPressed(Key::I, _) => {
                global.scroll_up();
            }
            InputEvent::KeyPressed(Key::K, _) => {
                global.scroll_down();
            }
            _ => {}
        }
    }
}