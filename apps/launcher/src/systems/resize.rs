use bevy_ecs::system::{Query, Res, ResMut};

use game_engine::render::{
    components::{Camera, Transform, Viewport},
    Window,
};

use crate::resources::Global;

pub fn handle_viewport_resize(
    global: Res<Global>,
    mut window: ResMut<Window>,
    mut cameras_q: Query<(&mut Camera, &mut Transform)>,
) {
    // sync camera viewport to window
    if !window.did_change() {
        return;
    }
    //info!("window did change");
    window.clear_change();
    let Some(window_res) = window.get() else {
        return;
    };
    //info!("window has res");

    if let Ok((mut camera, _transform)) = cameras_q.get_mut(global.camera_3d) {
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
        }
    }

    // resize scene camera
}
