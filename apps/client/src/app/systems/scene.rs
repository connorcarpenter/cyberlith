use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Local, Query, Res, ResMut},
};

use game_engine::{
    asset::{AnimationData, AssetManager, AssetHandle, ModelData},
    math::{Quat, Vec3},
    render::{
        base::{Color, CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
            OrthographicProjection, PointLight, Projection, RenderLayer, RenderLayers,
            RenderObjectBundle, RenderTarget, Transform, Viewport, Visibility,
        },
        resources::{RenderFrame, Time},
        shapes,
    },
    storage::{Handle, Storage},
};

#[derive(Component)]
pub struct WalkerMarker;

const ROOM_WIDTH: f32 = 300.0;
const ROOM_DEPTH: f32 = 300.0;
// const ROOM_HEIGHT: f32 = 200.0;

#[derive(Component)]
pub struct WalkAnimation {
    anim_handle: AssetHandle<AnimationData>,
    image_index: f32,
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
            mesh: meshes.add(shapes::Square),
            material: materials.add(CpuMaterial::new(Color::DARK_GRAY, 0.0, 0.0, 0.0)),
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
                viewport: Some(Viewport::new_at_origin(1280, 720)),
                order: 0,
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

pub fn scene_step(
    time: Res<Time>,
    asset_manager: Res<AssetManager>,
    mut object_q: Query<(&mut Transform, &mut WalkAnimation), With<WalkerMarker>>,
    mut rotation: Local<f32>,
) {
    let elapsed_time = time.get_elapsed();

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

        transform.rotate_x(0.001 * elapsed_time);
        transform.rotate_y(0.002 * elapsed_time);
        transform.rotation = Quat::from_rotation_z(f32::to_radians(*rotation + 90.0));

        // animate
        anim.image_index += 0.4 * elapsed_time;

        let subimage_count = asset_manager.get_animation_duration(&anim.anim_handle) as f32;

        while anim.image_index >= subimage_count {
            anim.image_index -= subimage_count;
        }
    }
}

pub fn scene_draw(
    asset_manager: Res<AssetManager>,
    mut render_frame: ResMut<RenderFrame>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // Meshes
    cpu_meshes_q: Query<(
        &Handle<CpuMesh>,
        &Handle<CpuMaterial>,
        &Transform,
        &Visibility,
        Option<&RenderLayer>,
    )>,
    models_q: Query<(
        &AssetHandle<ModelData>,
        &WalkAnimation,
        &Transform,
        &Visibility,
        Option<&RenderLayer>,
    )>,
    // Lights
    ambient_lights_q: Query<(&AmbientLight, Option<&RenderLayer>)>,
    point_lights_q: Query<(&PointLight, Option<&RenderLayer>)>,
    directional_lights_q: Query<(&DirectionalLight, Option<&RenderLayer>)>,
) {
    // Aggregate Cameras
    for (camera, transform, projection, render_layer_opt) in cameras_q.iter() {
        if !camera.is_active {
            continue;
        }
        render_frame.draw_camera(render_layer_opt, camera, transform, projection);
    }

    // Aggregate Point Lights
    for (point_light, render_layer_opt) in point_lights_q.iter() {
        render_frame.draw_point_light(render_layer_opt, point_light);
    }

    // Aggregate Directional Lights
    for (dir_light, render_layer_opt) in directional_lights_q.iter() {
        render_frame.draw_directional_light(render_layer_opt, dir_light);
    }

    // Aggregate Ambient Lights
    for (ambient_light, render_layer_opt) in ambient_lights_q.iter() {
        render_frame.draw_ambient_light(render_layer_opt, ambient_light);
    }

    // Aggregate Cpu Meshes
    for (mesh_handle, mat_handle, transform, visibility, render_layer_opt) in cpu_meshes_q.iter() {
        if !visibility.visible {
            continue;
        }
        render_frame.draw_mesh(render_layer_opt, mesh_handle, mat_handle, transform);
    }

    // Aggregate Models
    for (model_handle, walk_anim, transform, visibility, render_layer_opt) in models_q.iter() {
        if !visibility.visible {
            continue;
        }
        asset_manager.draw_animated_model(
            &mut render_frame,
            model_handle,
            &walk_anim.anim_handle,
            transform,
            walk_anim.image_index,
            render_layer_opt,
        );
    }
}
