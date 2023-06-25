use bevy_ecs::system::Query;

use math::{Vec2, Vec3};
use render_api::components::{Camera, OrthographicProjection, Projection, Transform, Viewport};

use crate::app::resources::global::Global;

pub fn update_2d_camera(
    global: &Global,
    texture_size: Vec2,
    camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
) {
    let Some(camera_entity) = global.camera_2d else {
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
    global: &Global,
    texture_size: Vec2,
    camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
) {
    let Some(camera_entity) = global.camera_3d else {
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
