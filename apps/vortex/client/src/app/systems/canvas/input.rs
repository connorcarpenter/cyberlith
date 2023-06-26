use bevy_ecs::system::{Query, Res, ResMut};
use bevy_log::info;

use input::{Input, Key};
use render_api::components::{Camera, Projection, Transform};

use crate::app::resources::canvas_state::CanvasState;

pub fn input(
    mut canvas_state: ResMut<CanvasState>,
    input: Res<Input>,
    mut camera_query: Query<(&mut Camera, &mut Transform, &mut Projection)>,
) {
    // check input
    if input.is_pressed(Key::S) {
        // disable 2d camera, enable 3d camera
        canvas_state.set_3d_mode(&mut camera_query);
    } else if input.is_pressed(Key::W) {
        // disable 3d camera, enable 2d camera
        canvas_state.set_2d_mode(&mut camera_query);
    }
}
