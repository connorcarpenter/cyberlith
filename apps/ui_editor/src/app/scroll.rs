use bevy_ecs::{
    event::EventReader,
    system::{Res, ResMut},
};

use game_engine::{
    asset::AssetManager,
    input::{InputEvent, Key},
    ui::UiManager,
};

use crate::app::examples::GlobalChatState;

pub fn scroll_events(
    mut global_chat_state: ResMut<GlobalChatState>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut input_events: EventReader<InputEvent>,
) {
    for event in input_events.read() {
        match event {
            InputEvent::KeyPressed(Key::I, _) => {
                global_chat_state.global_chat_scroll_up(&mut ui_manager, &asset_manager);
            }
            InputEvent::KeyPressed(Key::K, _) => {
                global_chat_state.global_chat_scroll_down(&mut ui_manager, &asset_manager);
            }
            _ => {}
        }
    }
}
