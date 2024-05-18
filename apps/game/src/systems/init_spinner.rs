
use bevy_ecs::system::{Local, Query, Res, ResMut};

use game_engine::{
    math::Vec3,
    render::{
        base::{Color, CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation,
            Projection, Transform, RenderLayer, Viewport,
        },
        resources::{Time, RenderFrame},
        shapes::UnitSquare,
    },
    storage::Storage,
};

pub fn draw(
    mut render_frame: ResMut<RenderFrame>,
    time: Res<Time>,
    mut meshes: ResMut<Storage<CpuMesh>>,
    mut materials: ResMut<Storage<CpuMaterial>>,
    cameras_q: Query<&Camera>,
    mut total_time_ms: Local<f32>,
) {
    *total_time_ms += time.get_elapsed_ms();

    let mut target_viewport: Viewport = Viewport::new_at_origin(0, 0);
    for camera in cameras_q.iter() {
        target_viewport = camera.viewport.unwrap();
    }

    let mut camera_bundle = CameraBundle::default_3d_perspective(&target_viewport);

    camera_bundle.camera.clear_operation = ClearOperation {
        red: None,
        green: None,
        blue: None,
        alpha: None,
        depth: Some(1.0),
    };

    let Projection::Perspective(perspective) = &camera_bundle.projection else {
        panic!("expected perspective projection");
    };
    let distance = ((target_viewport.height as f32) / 2.0)
        / f32::tan(perspective.fov / 2.0);
    let x = target_viewport.width as f32 * 0.5;
    let y = target_viewport.height as f32 * 0.5;
    camera_bundle.transform.translation.x = x;
    camera_bundle.transform.translation.y = y;
    camera_bundle.transform.translation.z = distance;
    camera_bundle.transform.look_at(Vec3::new(x, y, 0.0), Vec3::NEG_Y);

    render_frame.draw_camera(
        Some(&RenderLayer::UI),
        &camera_bundle.camera,
        &camera_bundle.transform,
        &camera_bundle.projection,
    );
    render_frame.draw_ambient_light(
        Some(&RenderLayer::UI),
        &AmbientLight::new(1.0, Color::WHITE),
    );

    let mesh_handle = meshes.add(UnitSquare);
    let mat_handle = materials.add(Color::WHITE);

    let mut spinner_transform = Transform::default();
    spinner_transform.scale.x = x * 0.618;
    spinner_transform.scale.y = y * 0.618;
    spinner_transform.translation.x = x - (spinner_transform.scale.x * 0.5);
    spinner_transform.translation.y = y - (spinner_transform.scale.y * 0.5);
    render_frame.draw_spinner(Some(&RenderLayer::UI), &mat_handle, &mesh_handle, &spinner_transform, *total_time_ms * 0.005);
}