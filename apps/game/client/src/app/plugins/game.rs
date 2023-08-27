use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Local, Query, Res, ResMut},
};

use math::Vec3;

use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{
        AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
        OrthographicProjection, Projection, RenderLayers, RenderObjectBundle, RenderTarget,
        Transform, Viewport,
    },
    resources::WindowSettings,
    shapes, Assets, Window,
};

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
            .add_systems(Update, rotate);
    }
}

fn setup(
    mut commands: Commands,
    window: Res<Window>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
) {
    let layer = RenderLayers::layer(0);

    // plane
    // commands.spawn(RenderObjectBundle {
    //     mesh: meshes.add(shapes::Square),
    //     material: materials.add(Color::from_rgb_f32(0.3, 0.5, 0.3)),
    //     transform: Transform::from_scale(Vec3::new(100.0, 1.0, 100.0)),
    //     ..Default::default()
    // });
    // cube
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(shapes::Cube),
            material: materials.add(Color::from_rgb_f32(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(50.0)),
            ..Default::default()
        })
        .insert(CubeMarker)
        .insert(layer);
    // light
    commands
        .spawn(AmbientLight {
            intensity: 0.1,
            color: Color::WHITE,
            ..Default::default()
        })
        .insert(layer);
    // commands.spawn(PointLight {
    //     position: Vec3::new(50.0, 50.0, 50.0),
    //     intensity: 5.0,
    //     color: Color::RED,
    //     ..Default::default()
    // }).insert(layer);
    commands
        .spawn(DirectionalLight {
            direction: Vec3::new(12.0, 45.0, 91.0),
            intensity: 2.0,
            color: Color::WHITE,
        })
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
            transform: Transform::from_xyz(100.0, 100.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Orthographic(
                OrthographicProjection {
                    height: 720.0,
                    near: -1000.0,
                    far: 1000.0,
                    ..Default::default()
                }, //  projection: Projection::Perspective(PerspectiveProjection {
                   //              fov: 45.0,
                   //              near: 0.0,
                   //              far: 500.0,
                   //              ..Default::default()
                   //          }
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
    let z = f32::to_radians(*rotation).sin() * 100.0;

    let mut transform = cube_q.single_mut();

    transform.translation.x = x;
    transform.translation.z = z;
}

fn rotate(mut query: Query<&mut Transform, With<CubeMarker>>) {
    for mut transform in &mut query {
        transform.rotate_x(0.015);
        transform.rotate_z(0.013);
        transform.rotate_y(0.011);
    }
}
