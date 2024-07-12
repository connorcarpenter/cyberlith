use bevy_ecs::{event::EventWriter, change_detection::ResMut, prelude::Query};

use game_engine::{
    render::{
        components::{Camera, Viewport},
        Window,
    },
};

use crate::{ui::events::{ResyncMatchLobbiesEvent, ResyncGlobalChatEvent, ResyncPublicUserInfoEvent}};

pub fn handle_viewport_resize(
    mut window: ResMut<Window>,
    mut resync_public_user_info_events: EventWriter<ResyncPublicUserInfoEvent>,
    mut resync_global_chat_events: EventWriter<ResyncGlobalChatEvent>,
    mut resync_match_lobbies_event: EventWriter<ResyncMatchLobbiesEvent>,
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

            resync_public_user_info_events.send(ResyncPublicUserInfoEvent);
            resync_global_chat_events.send(ResyncGlobalChatEvent::new(true));
            resync_match_lobbies_event.send(ResyncMatchLobbiesEvent);
        }
    }
}
