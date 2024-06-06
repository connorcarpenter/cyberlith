
use bevy_ecs::event::EventReader;

use game_engine::input::{InputEvent, Key};
use logging::info;

pub fn scroll_events(
    mut input_events: EventReader<InputEvent>,
) {
    for event in input_events.read() {
        match event {
            InputEvent::KeyPressed(Key::I, _) => {
                info!("Scroll Up");
            }
            InputEvent::KeyPressed(Key::K, _) => {
                info!("Scroll Down");
            }
            _ => {}
        }
    }
}