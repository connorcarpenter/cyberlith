use bevy_ecs::{change_detection::DetectChanges, system::{Query, Res}};

use math::{Vec2, Vec3};
use render_api::components::{Camera, OrthographicProjection, Projection, Transform, Viewport};

use crate::app::resources::canvas_state::CanvasState;

pub fn sync_all_cameras_visibility(
    canvas_state: Res<CanvasState>,
    mut camera_q: Query<&mut Camera>,
) {
    if !canvas_state.is_changed() {
        return;
    }

    canvas_state.update_cameras(&mut camera_q);
}

pub fn update_2d_camera(
    canvas_state: &CanvasState,
    texture_size: Vec2,
    camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
) {
    let Some(camera_entity) = canvas_state.camera_2d else {
        return;
    };
    let Ok((mut camera, mut transform, mut projection)) = camera_query.get_mut(camera_entity) else {
        return;
    };
    camera.viewport = Some(Viewport::new_at_origin(
        texture_size.x as u32,
        texture_size.y as u32,
    ));

    let center = texture_size * 0.5;

    *transform = Transform::from_xyz(center.x, center.y, -1.0)
        .looking_at(Vec3::new(center.x, center.y, 0.0), Vec3::NEG_Y);
    *projection = Projection::Orthographic(OrthographicProjection {
        height: texture_size.y,
        near: 0.0,
        far: 10.0,
    });
}

pub fn update_3d_camera(
    canvas_state: &CanvasState,
    texture_size: Vec2,
    camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
) {
    let Some(camera_entity) = canvas_state.camera_3d else {
        return;
    };
    let Ok((mut camera, _, _)) = camera_query.get_mut(camera_entity) else {
        return;
    };
    camera.viewport = Some(Viewport::new_at_origin(
        texture_size.x as u32,
        texture_size.y as u32,
    ));
}
