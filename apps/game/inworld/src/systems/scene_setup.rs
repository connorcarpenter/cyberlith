
use bevy_ecs::system::Commands;

use game_engine::{
    math::Vec3,
    render::{
        base::{Color, CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
            OrthographicProjection, Projection, RenderLayers, RenderObjectBundle, RenderTarget,
            Transform,
        },
        shapes,
    },
    storage::Storage,
    world::constants::{TILE_SIZE, TILE_COUNT},
};

const TILE_SCALE: f32 = (TILE_SIZE * 0.5) - 5.0;

pub fn scene_setup(
    commands: &mut Commands,
    meshes: &mut Storage<CpuMesh>,
    materials: &mut Storage<CpuMaterial>,
) {
    let layer = RenderLayers::layer(0);

    // spawn grid of floor tiles

    const NEG_TILE_NUM: i32 = -TILE_COUNT;
    const TOTAL_TILE_NUM: i32 = TILE_COUNT * 2 + 1;

    for tx in NEG_TILE_NUM..=TILE_COUNT {
        for ty in NEG_TILE_NUM..=TILE_COUNT {

            let x = tx as f32 * TILE_SIZE;
            let y = ty as f32 * TILE_SIZE;

            commands
                .spawn(RenderObjectBundle {
                    mesh: meshes.add(shapes::CenteredSquare),
                    material: materials.add(Color::DARK_GRAY),
                    transform: Transform::from_scale(Vec3::new(TILE_SCALE, TILE_SCALE, 1.0))
                        .with_translation(Vec3::new(x, y, 0.0)),
                    ..Default::default()
                })
                .insert(layer);
        }
    }

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
    const CAMERA_ANGLE: f32 = 60.0;
    const CAMERA_Z: f32 = 400.0;
    let camera_y: f32 = CAMERA_Z / CAMERA_ANGLE.to_radians().tan();
    let mut camera_y_scale: f32 = CAMERA_ANGLE.to_radians().sin();
    let mut camera_x_scale: f32 = 1.0;
    let mut camera_z_scale: f32 = 1.0;

    const SCREEN_HEIGHT: f32 = 960.0;
    const TOTAL_SCALE: f32 = (TILE_SIZE * (TOTAL_TILE_NUM as f32)) / SCREEN_HEIGHT;
    camera_x_scale *= TOTAL_SCALE;
    camera_y_scale *= TOTAL_SCALE;
    camera_z_scale *= TOTAL_SCALE;

    commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: None,
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Screen,
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, camera_y, CAMERA_Z)
                .with_scale(Vec3::new(camera_x_scale, camera_y_scale, camera_z_scale))
                .looking_at(Vec3::ZERO, Vec3::Z),
            projection: Projection::Orthographic(OrthographicProjection {
                near: 0.1,
                far: 10000.0,
                ..Default::default()
            }),
            //     Projection::Perspective(PerspectiveProjection {
            //                 fov: std::f32::consts::PI / 4.0,
            //                 near: 0.1,
            //                 far: 10000.0,
            //                }),
        })
        .insert(layer);
}
