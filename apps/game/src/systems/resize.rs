use bevy_ecs::{change_detection::ResMut, event::EventWriter, prelude::Query};

use game_engine::render::{
    components::{Camera, Viewport},
    Window,
};

use crate::ui::events::{
    ResyncLobbyGlobalEvent, ResyncLobbyUiEvent, ResyncUserUiEvent,
};

pub fn handle_viewport_resize(
    mut window: ResMut<Window>,
    mut resync_user_public_info_events: EventWriter<ResyncUserUiEvent>,
    mut resync_global_chat_events: EventWriter<ResyncLobbyGlobalEvent>,
    mut resync_match_lobbies_event: EventWriter<ResyncLobbyUiEvent>,
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

            resync_user_public_info_events.send(ResyncUserUiEvent);
            resync_global_chat_events.send(ResyncLobbyGlobalEvent::new(true));
            resync_match_lobbies_event.send(ResyncLobbyUiEvent);
        }
    }
}
