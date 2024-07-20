
use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Local, Query, Res},
};

use game_engine::{
    asset::{AnimationData, AssetHandle, AssetManager},
    math::{Quat, Vec3},
    render::{
        base::{Color, CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
            OrthographicProjection, Projection, RenderLayers, RenderObjectBundle, RenderTarget,
            Transform,
        },
        resources::Time,
        shapes,
    },
    storage::Storage,
    logging::info,
};

#[derive(Component)]
pub struct WalkerMarker;

const ROOM_WIDTH: f32 = 50.0;
const ROOM_DEPTH: f32 = 50.0;
// const ROOM_HEIGHT: f32 = 200.0;

#[derive(Component)]
pub struct WalkAnimation {
    pub(crate) anim_handle: AssetHandle<AnimationData>,
    pub(crate) animation_index_ms: f32,
}

impl WalkAnimation {
    pub fn new(anim_handle: AssetHandle<AnimationData>) -> Self {
        Self {
            anim_handle,
            animation_index_ms: 0.0,
        }
    }
}

pub fn scene_setup(
    commands: &mut Commands,
    meshes: &mut Storage<CpuMesh>,
    materials: &mut Storage<CpuMaterial>,
) {
    let layer = RenderLayers::layer(0);

    // plane

    // spawn grid of floor tiles
    const TILE_NUM: i32 = 5;
    const NEG_TILE_NUM: i32 = -TILE_NUM;
    const TOTAL_TILE_NUM: i32 = TILE_NUM * 2 + 1;

    for tx in NEG_TILE_NUM..=TILE_NUM {
        for ty in NEG_TILE_NUM..=TILE_NUM {

            let x = tx as f32 * ROOM_WIDTH * 2.0;
            let y = ty as f32 * ROOM_DEPTH * 2.0;

            commands
                .spawn(RenderObjectBundle {
                    mesh: meshes.add(shapes::CenteredSquare),
                    material: materials.add(Color::DARK_GRAY),
                    transform: Transform::from_scale(Vec3::new(ROOM_WIDTH - 5.0, ROOM_DEPTH - 5.0, 1.0))
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

    const SCREEN_HEIGHT: f32 = 1000.0;
    const TOTAL_SCALE: f32 = (ROOM_DEPTH * 2.0 * (TOTAL_TILE_NUM as f32)) / SCREEN_HEIGHT;
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

pub fn step(
    time: Res<Time>,
    asset_manager: Res<AssetManager>,
    mut object_q: Query<(&mut Transform, &mut WalkAnimation), With<WalkerMarker>>,
    mut rotation: Local<f32>,
    mut rotation_stepped: Local<f32>,
    mut animation_index_ms: Local<f32>,
) {
    let elapsed_time = time.get_elapsed_ms();

    if *rotation == 0.0 {
        *rotation = 0.01;
    } else {
        *rotation += 0.03 * elapsed_time;
        if *rotation > 359.0 {
            *rotation = 0.01;
        }

        // step angles at 45 degree increments
        *rotation_stepped = (*rotation / 45.0).round() * 45.0;
        if *rotation_stepped == 360.0 {
            *rotation_stepped = 0.0;
        }

    }

    for (mut transform, mut anim) in object_q.iter_mut() {
        // rotate
        // transform.translation.x = 0.0; // f32::to_radians(*rotation).cos() * 250.0;
        // transform.translation.y = 0.0; // f32::to_radians(*rotation).sin() * 250.0;
        // transform.translation.z = 0.0;

        // transform.rotate_x(0.001 * elapsed_time);
        // transform.rotate_y(0.002 * elapsed_time);
        transform.rotation = Quat::from_rotation_z(f32::to_radians(*rotation_stepped));

        // animate
        *animation_index_ms += 0.4 * elapsed_time;

        let animation_duration_ms = asset_manager.get_animation_duration_ms(&anim.anim_handle) as f32;
        while *animation_index_ms >= animation_duration_ms {
            *animation_index_ms -= animation_duration_ms;
        }

        let avg_frame_ms = 40.0;

        // round to the nearest image index
        anim.animation_index_ms = (*animation_index_ms / avg_frame_ms).round() * avg_frame_ms;
        if anim.animation_index_ms >= animation_duration_ms {
            anim.animation_index_ms = 0.0;
        }

        // info!("anim.image_index: {:?} / subimage_count: {:?}", anim.animation_index_ms, animation_duration_ms);
    }
}
