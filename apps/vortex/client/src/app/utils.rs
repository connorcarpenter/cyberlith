use bevy_ecs::{entity::Entity, system::Query};

use math::{convert_2d_to_3d, quat_from_spin_direction, Quat, Vec2, Vec3};
use render_api::components::{Camera, CameraProjection, Projection, Transform};

use crate::app::resources::camera_manager::CameraManager;

pub fn set_3d_line_transform(transform: &mut Transform, start: Vec3, end: Vec3, spin: f32) {
    transform.translation = start;

    if start == end {
        transform.scale.x = 0.0;
        return;
    }

    if start.y == end.y && start.z == end.z {
        let forward = end.x - start.x;
        if forward > 0.0 {
            transform.rotation = Quat::IDENTITY;
        } else {
            transform.rotation = Quat::from_rotation_z(f32::to_radians(180.0));
        }
    } else {
        let rotation_angle = quat_from_spin_direction(spin, Vec3::X, end - start);
        transform.rotation = rotation_angle;
    }

    transform.scale.x = start.distance(end);
}

// spin is in radians
pub fn transform_from_endpoints_and_spin(start: Vec3, end: Vec3, spin: f32) -> Transform {
    let mut output = Transform::default();
    output.translation = start;

    if start.y == end.y && start.z == end.z {
        output.look_to(end - start, Vec3::Z);
    } else {
        let rotation_angle = quat_from_spin_direction(spin, Vec3::X, end - start);
        output.rotation = rotation_angle;
    }

    output
}

pub fn get_new_3d_position(
    camera_manager: &CameraManager,
    camera_q: &Query<(&Camera, &Projection)>,
    transform_q: &Query<&Transform>,
    mouse_position: &Vec2,
    entity_2d: &Entity,
) -> Vec3 {
    let camera_3d = camera_manager.camera_3d_entity().unwrap();
    let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
    let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();

    let camera_viewport = camera.viewport.unwrap();
    let view_matrix = camera_transform.view_matrix();
    let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

    // get 2d shape transform
    let transform_2d = transform_q.get(*entity_2d).unwrap();

    // convert 2d to 3d
    let new_3d_position = convert_2d_to_3d(
        &view_matrix,
        &projection_matrix,
        &camera_viewport.size_vec2(),
        &mouse_position,
        transform_2d.translation.z,
    );
    new_3d_position
}
