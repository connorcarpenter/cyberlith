use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Local, Query, Res, ResMut},
};

use game_engine::{
    math::Vec3,
    render::{
        base::{Color, CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
            PerspectiveProjection, Projection, RenderLayers, RenderObjectBundle, RenderTarget,
            Transform,
        },
        resources::Time,
        shapes,
    },
    storage::Storage,
};

#[derive(Component)]
pub struct CubeMarker;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Storage<CpuMesh>>,
    mut materials: ResMut<Storage<CpuMaterial>>,
) {
    // render_layer
    let layer = RenderLayers::layer(0);

    // cube
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(shapes::Cube),
            material: materials.add(Color::RED),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 120.0))
                .with_scale(Vec3::splat(30.0)),
            ..Default::default()
        })
        .insert(CubeMarker)
        .insert(layer);

    // ambient light
    commands
        .spawn(AmbientLight::new(0.1, Color::WHITE))
        .insert(layer);

    // directional light
    let light_source = Vec3::new(-500.0, 500.0, 200.0);
    let light_target = Vec3::ZERO;
    commands
        .spawn(DirectionalLight::new(
            2.0,
            Color::WHITE,
            light_target - light_source,
        ))
        .insert(layer);

    // camera
    commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: None,
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Screen,
                ..Default::default()
            },
            transform: Transform::from_xyz(400.0, 400.0, 400.0).looking_at(Vec3::ZERO, Vec3::Z),
            projection:
            // Projection::Orthographic(OrthographicProjection {
            //     near: 0.1,
            //     far: 10000.0,
            //     ..Default::default()
            // }),
            Projection::Perspective(PerspectiveProjection {
                fov: std::f32::consts::PI / 4.0,
                near: 0.1,
                far: 10000.0,
            }),
        })
        .insert(layer);
}

pub fn step(
    time: Res<Time>,
    mut object_q: Query<&mut Transform, With<CubeMarker>>,
    mut rotation: Local<f32>,
) {
    let elapsed_time = time.get_elapsed_ms();

    if *rotation == 0.0 {
        *rotation = 0.01;
    } else {
        *rotation += 0.03 * elapsed_time;
        if *rotation > 359.0 {
            *rotation = 0.01;
        }
    }

    for mut transform in object_q.iter_mut() {
        // rotate
        transform.translation.x = f32::to_radians(*rotation).cos() * 250.0;
        transform.translation.y = f32::to_radians(*rotation).sin() * 250.0;
        transform.translation.z = 60.0;

        transform.rotate_x(0.001 * elapsed_time);
        transform.rotate_y(0.002 * elapsed_time);
    }
}
