use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Local, Query, Res, ResMut},
};

use game_engine::{
    asset::{AnimationData, AssetHandle, AssetManager},
    math::{Quat, Vec3},
    render::{
        base::{Color, CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
            OrthographicProjection, Projection, RenderLayers,
            RenderObjectBundle, RenderTarget, Transform,
        },
        resources::Time,
        shapes,
    },
    storage::Storage,
};

#[derive(Component)]
pub struct WalkerMarker;

const ROOM_WIDTH: f32 = 300.0;
const ROOM_DEPTH: f32 = 300.0;
// const ROOM_HEIGHT: f32 = 200.0;

#[derive(Component)]
pub struct WalkAnimation {
    pub(crate) anim_handle: AssetHandle<AnimationData>,
    pub(crate) image_index: f32,
}

impl WalkAnimation {
    pub fn new(anim_handle: AssetHandle<AnimationData>) -> Self {
        Self {
            anim_handle,
            image_index: 0.0,
        }
    }
}

pub fn scene_setup(
    mut commands: Commands,
    mut meshes: ResMut<Storage<CpuMesh>>,
    mut materials: ResMut<Storage<CpuMaterial>>,
) {
    let layer = RenderLayers::layer(0);

    // plane
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(shapes::CenteredSquare),
            material: materials.add(Color::DARK_GRAY),
            transform: Transform::from_scale(Vec3::new(ROOM_WIDTH, ROOM_DEPTH, 1.0))
                .with_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
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

    for (mut transform, mut anim) in object_q.iter_mut() {
        // rotate
        transform.translation.x = f32::to_radians(*rotation).cos() * 250.0;
        transform.translation.y = f32::to_radians(*rotation).sin() * 250.0;
        transform.translation.z = 0.0;

        // transform.rotate_x(0.001 * elapsed_time);
        // transform.rotate_y(0.002 * elapsed_time);
        transform.rotation = Quat::from_rotation_z(f32::to_radians(*rotation + 90.0));

        // animate
        anim.image_index += 0.4 * elapsed_time;

        let subimage_count = asset_manager.get_animation_duration(&anim.anim_handle) as f32;

        while anim.image_index >= subimage_count {
            anim.image_index -= subimage_count;
        }
    }
}