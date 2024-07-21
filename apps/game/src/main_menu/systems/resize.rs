use bevy_ecs::event::{EventReader, EventWriter};

use crate::{systems::resize::ViewportResizeEvent, main_menu::ui::events::{ResyncLobbyListUiEvent, ResyncMessageListUiEvent, ResyncUserListUiEvent}};

pub fn resync_on_resize(
    mut resize_events: EventReader<ViewportResizeEvent>,
    mut resync_user_ui_events: EventWriter<ResyncUserListUiEvent>,
    mut resync_chat_events: EventWriter<ResyncMessageListUiEvent>,
    mut resync_lobby_ui_events: EventWriter<ResyncLobbyListUiEvent>,
) {
    let mut resize = false;
    for _ in resize_events.read() {
        resize = true;
    }
    if resize {
        resync_user_ui_events.send(ResyncUserListUiEvent);
        resync_chat_events.send(ResyncMessageListUiEvent::new(true));
        resync_lobby_ui_events.send(ResyncLobbyListUiEvent);
    }
}