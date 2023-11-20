use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Local, Query, Res, ResMut},
};

use asset::MeshFile;
use math::{Quat, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{
        AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
        OrthographicProjection, PointLight, Projection, RenderLayer, RenderLayers,
        RenderObjectBundle, RenderTarget, Transform, Viewport, Visibility,
    },
    resources::{RenderFrame, Time, WindowSettings},
    shapes, Assets, Handle,
};
use render_api::components::PerspectiveProjection;
use render_api::shapes::Sphere;

#[derive(Component)]
pub struct CubeMarker;

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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut ambient_lights: ResMut<Assets<AmbientLight>>,
    mut dir_lights: ResMut<Assets<DirectionalLight>>,
) {
    let layer = RenderLayers::layer(0);

    // load assets
    //let file_cube_mesh = MeshFile::load("cube.mesh");
    //let file_cube_mesh_handle = meshes.add(file_cube_mesh);
    let sphere_mesh_handle = meshes.add(Sphere::new(10));
    let sphere_mesh_handle2 = meshes.add(Sphere::new(8));

    let red_mat_handle = materials.add(Color::from_rgb_f32(1.0, 0.0, 0.0));
    let blue_mat_handle = materials.add(Color::from_rgb_f32(0.0, 0.0, 1.0));

    // plane
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(shapes::Square),
            material: materials.add(Color::from_rgb_f32(0.5, 0.5, 0.5)),
            transform: Transform::from_scale(Vec3::new(300.0, 300.0, 1.0))
                .with_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        //.insert(CubeMarker)
        .insert(layer);
    // top left cube (RED)
    commands
        .spawn(RenderObjectBundle {
            mesh: sphere_mesh_handle,
            material: red_mat_handle,
            transform: Transform::from_scale(Vec3::splat(50.0))
                .with_translation(Vec3::new(0.0, 0.0, 70.0))
                .with_rotation(Quat::from_axis_angle(Vec3::X, f32::to_radians(90.0))),
            ..Default::default()
        })
        //.insert(CubeMarker)
        .insert(layer);
    // top right cube
    commands
        .spawn(RenderObjectBundle {
            mesh: sphere_mesh_handle2,
            material: blue_mat_handle,
            transform: Transform::from_scale(Vec3::splat(30.0))
                .with_translation(Vec3::new(100.0, 0.0, 70.0))
                .with_rotation(Quat::from_axis_angle(Vec3::X, f32::to_radians(90.0))),
            ..Default::default()
        })
        .insert(layer);
    // bottom left cube
    commands
        .spawn(RenderObjectBundle {
            mesh: sphere_mesh_handle2,
            material: blue_mat_handle,
            transform: Transform::from_scale(Vec3::splat(30.0))
                .with_translation(Vec3::new(0.0, 100.0, 70.0))
                .with_rotation(Quat::from_axis_angle(Vec3::X, f32::to_radians(90.0))),
            ..Default::default()
        })
        .insert(layer);
    // ambient light
    commands
        .spawn(ambient_lights.add(AmbientLight::new(0.1, Color::WHITE)))
        .insert(layer);
    // directional light
    // let light_source = Vec3::new(0.0, 500.0, 10000.0);
    // let light_target = Vec3::ZERO;
    // commands
    //     .spawn(dir_lights.add(DirectionalLight::new(
    //         2.0,
    //         Color::WHITE,
    //         light_target - light_source,
    //     )))
    //     .insert(layer);
    // point light
    commands.spawn(PointLight::new(Vec3::new(0.0, 0.0, 100.0), 3.0, Color::WHITE, Default::default()))
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
    mut cube_q: Query<&mut Transform, With<CubeMarker>>,
    mut light_q: Query<&mut PointLight>,
    mut rotation: Local<f32>,
) {
    //info!("elapsed time: {}", frame_input.elapsed_time);

    let elapsed_time = (time.get_elapsed() / 16.0) as f32;

    if *rotation == 0.0 {
        *rotation = 0.01;
    } else {
        *rotation += 1.0 * elapsed_time;
        if *rotation > 359.0 {
            *rotation = 0.01;
        }
    }

    let x = f32::to_radians(*rotation).cos() * 300.0;
    let y = f32::to_radians(*rotation).sin() * 300.0;

    // for mut transform in cube_q.iter_mut() {
    //     // rotate position
    //     transform.translation.x = x;
    //     transform.translation.y = y;
    //
    //     // rotate model
    //     transform.rotate_x(0.01 * elapsed_time);
    //     transform.rotate_y(0.02 * elapsed_time);
    // }

    for mut point_light in light_q.iter_mut() {
        point_light.position.x = x;
        point_light.position.y = y;
        point_light.position.z = 70.0;
    }
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
    ambient_lights_q: Query<(&Handle<AmbientLight>, Option<&RenderLayer>)>,
    point_lights_q: Query<(&PointLight, Option<&RenderLayer>)>,
    directional_lights_q: Query<(&Handle<DirectionalLight>, Option<&RenderLayer>)>,
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
    for (handle, render_layer_opt) in directional_lights_q.iter() {
        render_frame.draw_directional_light(render_layer_opt, handle);
    }

    // Aggregate Ambient Lights
    for (handle, render_layer_opt) in ambient_lights_q.iter() {
        render_frame.draw_ambient_light(render_layer_opt, handle);
    }

    // Aggregate RenderObjects
    for (mesh_handle, mat_handle, transform, visibility, render_layer_opt) in objects_q.iter() {
        if !visibility.visible {
            continue;
        }
        render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);
    }
}
