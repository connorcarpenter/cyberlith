use bevy_ecs::{
    event::EventReader,
    system::{Res, ResMut},
};

use game_engine::{
    asset::AssetManager,
    input::{InputEvent, Key},
    ui::UiManager,
};

use crate::app::global::Global;

pub fn scroll_events(
    mut global: ResMut<Global>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut input_events: EventReader<InputEvent>,
) {
    for event in input_events.read() {
        match event {
            InputEvent::KeyPressed(Key::I, _) => {
                global.scroll_up(&mut ui_manager, &asset_manager);
            }
            InputEvent::KeyPressed(Key::K, _) => {
                global.scroll_down(&mut ui_manager, &asset_manager);
            }
            _ => {}
        }
    }
}
