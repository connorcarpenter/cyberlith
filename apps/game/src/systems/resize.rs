use bevy_ecs::{change_detection::ResMut, event::EventWriter, prelude::Query};

use game_engine::render::{
    components::{Camera, Viewport},
    Window,
};

use crate::main_menu::ui::events::{ResyncLobbyListUiEvent, ResyncMessageListUiEvent, ResyncUserListUiEvent};

pub fn handle_viewport_resize(
    mut window: ResMut<Window>,
    mut resync_user_ui_events: EventWriter<ResyncUserListUiEvent>,
    mut resync_chat_events: EventWriter<ResyncMessageListUiEvent>,
    mut resync_lobby_ui_events: EventWriter<ResyncLobbyListUiEvent>,
    mut cameras_q: Query<&mut Camera>,
) {
    // sync camera viewport to window
    if !window.did_change() {
        return;
    }
    window.clear_change();
    let Some(window_res) = window.get() else {
        return;
    };
    for mut camera in cameras_q.iter_mut() {
        let should_change = if let Some(viewport) = camera.viewport.as_mut() {
            *viewport != window_res.logical_size
        } else {
            true
        };
        if should_change {
            let new_viewport = Viewport::new_at_origin(
                window_res.logical_size.width,
                window_res.logical_size.height,
            );
            camera.viewport = Some(new_viewport);

            //info!("resize window detected. new viewport: (x: {:?}, y: {:?}, width: {:?}, height: {:?})", new_viewport.x, new_viewport.y, new_viewport.width, new_viewport.height);

            resync_user_ui_events.send(ResyncUserListUiEvent);
            resync_chat_events.send(ResyncMessageListUiEvent::new(true));
            resync_lobby_ui_events.send(ResyncLobbyListUiEvent);
        }
    }
}
