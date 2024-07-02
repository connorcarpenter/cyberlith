use bevy_ecs::{change_detection::ResMut, prelude::Query, system::Res};

use game_engine::{
    asset::AssetManager,
    render::{
        components::{Camera, Viewport},
        Window,
    },
    ui::UiManager,
    session::components::{GlobalChatMessage, PresentUserInfo},
};

use crate::resources::{user_presence::UserPresence, global_chat::GlobalChat};

pub fn handle_viewport_resize(
    mut window: ResMut<Window>,
    mut global_chat: ResMut<GlobalChat>,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    user_presence: Res<UserPresence>,
    mut cameras_q: Query<&mut Camera>,
    user_q: Query<&PresentUserInfo>,
    message_q: Query<&GlobalChatMessage>,
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

            global_chat.sync_with_collection(&mut ui_manager, &asset_manager, &user_presence, &user_q, &message_q);
        }
    }
}
