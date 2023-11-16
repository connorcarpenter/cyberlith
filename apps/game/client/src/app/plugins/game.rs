use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Local, Query, Res, ResMut},
};

use math::{Quat, Vec3};

use render_api::{base::{Color, CpuMaterial, CpuMesh}, components::{
    AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
    OrthographicProjection, Projection, RenderLayers, RenderObjectBundle, RenderTarget,
    Transform, Viewport,
}, resources::WindowSettings, shapes, Assets, Handle};
use render_api::components::{PerspectiveProjection, PointLight, RenderLayer, Visibility};
use render_api::resources::RenderFrame;

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
            .add_systems(Update, rotate)
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

    // plane
    commands.spawn(RenderObjectBundle {
        mesh: meshes.add(shapes::Square),
        material: materials.add(Color::from_rgb_f32(0.5, 0.5, 0.5)),
        transform: Transform::from_scale(Vec3::new(300.0, 300.0, 1.0)).with_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    })
        //.insert(CubeMarker)
        .insert(layer);
    // top left cube (RED)
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(shapes::Cube),
            material: materials.add(Color::from_rgb_f32(1.0, 0.0, 0.0)),
            transform: Transform::from_scale(Vec3::splat(50.0)).with_translation(Vec3::new(-70.0, -70.0, 70.0)),
            ..Default::default()
        })
        .insert(CubeMarker)
        .insert(layer);
    // top right cube (GREEN)
    // commands
    //     .spawn(RenderObjectBundle {
    //         mesh: meshes.add(shapes::Cube),
    //         material: materials.add(Color::from_rgb_f32(0.0, 1.0, 0.0)),
    //         transform: Transform::from_scale(Vec3::splat(50.0)).with_translation(Vec3::new(70.0, -70.0, 70.0)),
    //         ..Default::default()
    //     })
    //     .insert(layer);
    // bottom left cube (BLUE)
    // commands
    //     .spawn(RenderObjectBundle {
    //         mesh: meshes.add(shapes::Cube),
    //         material: materials.add(Color::from_rgb_f32(0.0, 0.0, 1.0)),
    //         transform: Transform::from_scale(Vec3::splat(50.0)).with_translation(Vec3::new(-70.0, 70.0, 70.0)),
    //         ..Default::default()
    //     })
    //     .insert(layer);
    // ambient light
    commands
        .spawn(ambient_lights.add(AmbientLight::new(0.1, Color::WHITE)))
        .insert(layer);
    // directional light
    let light_source = Vec3::new(500.0, 250.0, 1000.0);
    let light_target = Vec3::ZERO;
    commands
        .spawn(dir_lights.add(DirectionalLight::new(2.0, Color::WHITE, light_target - light_source)))
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
            transform: Transform::from_xyz(250.0, 500.0, 500.0).looking_at(Vec3::ZERO, Vec3::Z),
            // projection: Projection::Orthographic(
            //     OrthographicProjection {
            //         near: 0.1,
            //         far: 10000.0,
            //         ..Default::default()
            //     },
                 projection: Projection::Perspective(PerspectiveProjection {
                             fov: std::f32::consts::PI / 4.0,
                             near: 0.1,
                             far: 10000.0,
                            }
            ),
        })
        .insert(layer);
}

fn step(mut cube_q: Query<&mut Transform, With<CubeMarker>>, mut rotation: Local<f32>) {
    if *rotation == 0.0 {
        *rotation = 0.01;
    } else {
        *rotation += 1.0;
        if *rotation > 359.0 {
            *rotation = 0.01;
        }
    }

    let x = f32::to_radians(*rotation).cos() * 100.0;
    let y = f32::to_radians(*rotation).sin() * 100.0;

    for mut transform in cube_q.iter_mut() {
        transform.translation.x = x;
        transform.translation.y = y;
    }
}

fn rotate(mut query: Query<&mut Transform, With<CubeMarker>>) {
    for mut transform in &mut query {
        transform.rotate_x(0.01);
        transform.rotate_y(0.02);
        //transform.rotate_y(0.011);
    }
}

pub fn draw(
    mut render_frame: ResMut<RenderFrame>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // Objects
    objects_q: Query<
        (
            &Handle<CpuMesh>,
            &Handle<CpuMaterial>,
            &Transform,
            &Visibility,
            Option<&RenderLayer>,
        )
    >,
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