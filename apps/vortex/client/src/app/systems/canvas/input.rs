use bevy_ecs::system::{Query, Res};

use input::{Input, Key};
use render_api::components::{Camera, Projection, Transform};

use crate::app::resources::canvas_state::CanvasState;

pub fn input(
    canvas_state: Res<CanvasState>,
    input: Res<Input>,
    mut camera_query: Query<(&mut Camera, &mut Transform, &mut Projection)>,
) {
    // check input
    if input.is_pressed(Key::S) {
        // disable 2d camera, enable 3d camera
        enable_cameras(&canvas_state, &mut camera_query, false, true);
    } else if input.is_pressed(Key::W) {
        // disable 3d camera, enable 2d camera
        enable_cameras(&canvas_state, &mut camera_query, true, false);
    }
}

fn enable_cameras(
    canvas_state: &CanvasState,
    camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    enable_2d: bool,
    enable_3d: bool,
) {
    if let Some(camera_2d) = canvas_state.camera_2d {
        if let Ok((mut camera, _, _)) = camera_query.get_mut(camera_2d) {
            camera.is_active = enable_2d;
        };
    }
    if let Some(camera_3d) = canvas_state.camera_3d {
        if let Ok((mut camera, _, _)) = camera_query.get_mut(camera_3d) {
            camera.is_active = enable_3d;
        };
    }
}
