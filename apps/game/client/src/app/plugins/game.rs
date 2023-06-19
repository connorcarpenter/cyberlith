use bevy_app::{App, Plugin};
use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Local, Query, Res, ResMut},
};

use math::Vec3;
use render_api::{
    Assets,
    base::{Color, PbrMaterial, TriMesh},
    components::{
        AmbientLight, Camera, CameraBundle, DirectionalLight, PerspectiveProjection, Projection,
        RenderObjectBundle, Transform,
    },
    resources::WindowSettings, shapes, Window,
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
            .add_startup_system(setup)
            // Receive Client Events
            // .add_systems(
            //     (
            //         network::connect_events,
            //         network::disconnect_events,
            //         network::reject_events,
            //         network::error_events,
            //     )
            //         .chain()
            //         .in_set(ReceiveEvents),
            // )
            .add_system(step)
            .add_system(rotate);
    }
}

fn setup(
    mut commands: Commands,
    window: Res<Window>,
    mut meshes: ResMut<Assets<TriMesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    // plane
    commands.spawn(RenderObjectBundle {
        mesh: meshes.add(shapes::Rectangle::from_size(50.0).into()),
        material: materials.add(Color::from_rgb_f32(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(TriMesh::from(shapes::Cube { size: 10.0 })),
            material: materials.add(Color::from_rgb_f32(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 10.0, 0.0),
            ..Default::default()
        })
        .insert(CubeMarker);
    // light
    commands.spawn(AmbientLight {
        intensity: 0.1,
        color: Color::WHITE,
        ..Default::default()
    });
    // commands.spawn(PointLight {
    //     position: Vec3::new(40.0, 80.0, 40.0),
    //     intensity: 0.3,
    //     color: Color::RED,
    //     ..Default::default()
    // });
    commands.spawn(DirectionalLight {
        direction: Vec3::new(0.0, -1.0, -2.0),
        intensity: 1.0,
        color: Color::WHITE,
    });
    // camera
    commands.spawn(CameraBundle {
        camera: Camera {
            viewport: Some(window.viewport()),
            ..Default::default()
        },
        transform: Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 45.0,
            near: 0.1,
            far: 1000.0,
            ..Default::default()
        }),
    });
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

    let x = f32::to_radians(*rotation).cos() * 10.0;
    let z = f32::to_radians(*rotation).sin() * 10.0;

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
