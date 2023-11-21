use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Local, Query, Res, ResMut},
};
use rand::Rng;

use math::{Quat, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{
        AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
        OrthographicProjection, PerspectiveProjection, PointLight, Projection, RenderLayer,
        RenderLayers, RenderObjectBundle, RenderTarget, Transform, Viewport, Visibility,
    },
    resources::{RenderFrame, Time, WindowSettings},
    shapes, Assets, Handle,
};

#[derive(Component)]
pub struct BallMarker;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Cyberlith".to_string(),
                max_size: Some((1280, 720)),
                ..Default::default()
            })
            // Add Naia Client Plugin
            // .add_plugin(NaiaClientPlugin::new(
            //     NaiaClientConfig::default(),
            //     protocol(),
            // ))
            // Startup Systems
            // .add_startup_system(network::init)
            .add_systems(Startup, setup)
            // Receive Client Events
            // .add_systems(
            //     (
            //         network::connect_events,
            //         network::disconnect_events,
            //         network::reject_events,
            //         network::error_events,
            //     )
            //         .in_set(ReceiveEvents),
            // )
            .add_systems(Update, step)
            .add_systems(Update, draw);
    }
}

const ROOM_WIDTH: f32 = 300.0;
const ROOM_DEPTH: f32 = 300.0;
const ROOM_HEIGHT: f32 = 200.0;
const GRAVITY: f32 = 0.1;
const BALL_COUNT: u32 = 1000;
const BALL_SIZE: f32 = 10.0;
const MESH_COUNT: u32 = 100;
const MAX_VELOCITY: f32 = 20.0;

#[derive(Component)]
struct Velocity(Vec3);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
) {
    let layer = RenderLayers::layer(0);

    let mut sphere_mesh_handles = Vec::new();
    for i in 0..MESH_COUNT {
        sphere_mesh_handles.push(meshes.add(shapes::Sphere::new((i + 4) as u16)));
    }

    let red_mat_handle = materials.add(Color::from_rgb_f32(1.0, 0.0, 0.0));

    let mut rng = rand::thread_rng();

    // ballz
    let mut mesh_index = 0;
    for _ in 0..BALL_COUNT {

        let x = rng.gen_range(-ROOM_WIDTH .. ROOM_WIDTH);
        let y = rng.gen_range(-ROOM_DEPTH .. ROOM_DEPTH);
        let z = rng.gen_range(BALL_SIZE .. ROOM_HEIGHT);

        let vx = rng.gen_range(-2.0 .. 2.0);
        let vy = rng.gen_range(-2.0 .. 2.0);
        let vz = rng.gen_range(-1.0 .. 1.0);

        commands
            .spawn(RenderObjectBundle {
                //mesh: meshes.add(shapes::Cube),
                mesh: sphere_mesh_handles[mesh_index],
                material: red_mat_handle,
                transform: Transform::from_scale(Vec3::splat(BALL_SIZE))
                    .with_translation(Vec3::new(x,y,z)),
                ..Default::default()
            })
            .insert(BallMarker)
            .insert(Velocity(Vec3::new(vx,vy,vz)))
            .insert(layer);

        mesh_index += 1;
        if mesh_index >= sphere_mesh_handles.len() {
            mesh_index = 0;
        }
    }

    // plane
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(shapes::Square),
            material: materials.add(Color::from_rgb_f32(0.5, 0.5, 0.5)),
            transform: Transform::from_scale(Vec3::new(ROOM_WIDTH, ROOM_DEPTH, 1.0))
                .with_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        //.insert(CubeMarker)
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
    // point light
    // commands.spawn(PointLight::new(Vec3::new(0.0, 0.0, 100.0), 3.0, Color::WHITE, Default::default()))
    //     .insert(layer);

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
            transform: Transform::from_xyz(0.0, 500.0, 500.0).looking_at(Vec3::ZERO, Vec3::Z),
            projection:
            // Projection::Orthographic(
            //     OrthographicProjection {
            //         near: 0.1,
            //         far: 10000.0,
            //         ..Default::default()
            //     }),
                Projection::Perspective(PerspectiveProjection {
                            fov: std::f32::consts::PI / 4.0,
                            near: 0.1,
                            far: 10000.0,
                           }),
        })
        .insert(layer);
}

fn step(
    time: Res<Time>,
    mut ball_q: Query<(&mut Velocity, &mut Transform), With<BallMarker>>,
    // mut light_q: Query<&mut PointLight>,
    mut rotation: Local<f32>,
) {
    //info!("elapsed time: {}", frame_input.elapsed_time);

    let elapsed_time = (time.get_elapsed() / 16.0) as f32;

    // if *rotation == 0.0 {
    //     *rotation = 0.01;
    // } else {
    //     *rotation += 1.0 * elapsed_time;
    //     if *rotation > 359.0 {
    //         *rotation = 0.01;
    //     }
    // }

    for (mut velocity, mut transform) in ball_q.iter_mut() {
        transform.translation.x += velocity.0.x;
        transform.translation.y += velocity.0.y;
        transform.translation.z += velocity.0.z;

        velocity.0.z -= GRAVITY;

        if transform.translation.z < BALL_SIZE {
            velocity.0.z = -velocity.0.z;
            transform.translation.z = BALL_SIZE;
        }

        // keep within room X, Y
        if transform.translation.x < -ROOM_WIDTH + BALL_SIZE {
            velocity.0.x = -velocity.0.x;
            transform.translation.x = -ROOM_WIDTH + BALL_SIZE;
        }
        if transform.translation.x > ROOM_WIDTH - BALL_SIZE {
            velocity.0.x = -velocity.0.x;
            transform.translation.x = ROOM_WIDTH - BALL_SIZE;
        }
        if transform.translation.y < -ROOM_DEPTH + BALL_SIZE {
            velocity.0.y = -velocity.0.y;
            transform.translation.y = -ROOM_DEPTH + BALL_SIZE;
        }
        if transform.translation.y > ROOM_DEPTH - BALL_SIZE {
            velocity.0.y = -velocity.0.y;
            transform.translation.y = ROOM_DEPTH - BALL_SIZE;
        }

        if velocity.0.z > MAX_VELOCITY {
            velocity.0.z = MAX_VELOCITY;
        }
    }

    // for mut point_light in light_q.iter_mut() {
    //     point_light.position.x = x;
    //     point_light.position.y = y;
    //     point_light.position.z = 200.0;
    // }
}

pub fn draw(
    mut render_frame: ResMut<RenderFrame>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // Objects
    objects_q: Query<(
        &Handle<CpuMesh>,
        &Handle<CpuMaterial>,
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

    // Aggregate RenderObjects
    for (mesh_handle, mat_handle, transform, visibility, render_layer_opt) in objects_q.iter() {
        if !visibility.visible {
            continue;
        }
        render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);
    }
}
