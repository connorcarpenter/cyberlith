use bevy_app::{App, Plugin};
use bevy_ecs::{
    component::Component,
    query::With,
    schedule::IntoSystemConfigs,
    system::{Commands, Local, Query, Res, ResMut},
};

use naia_bevy_client::{
    ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin, ReceiveEvents,
};

use math::Vec3;
use render_api::{
    base::{Camera, Color, PbrMaterial, TriMesh, Viewport},
    shape, AmbientLight, Assets, CameraComponent, ClearOperation, DirectionalLight, PointLight,
    RenderObjectBundle, RenderTarget, Transform, Window,
};

use game_proto::protocol;

use crate::app::network;

#[derive(Component)]
pub struct CubeMarker;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Add Naia Client Plugin
            .add_plugin(NaiaClientPlugin::new(
                NaiaClientConfig::default(),
                protocol(),
            ))
            // Startup Systems
            .add_startup_system(network::init)
            .add_startup_system(setup)
            // Receive Client Events
            .add_systems(
                (
                    network::connect_events,
                    network::disconnect_events,
                    network::reject_events,
                    network::error_events,
                )
                    .chain()
                    .in_set(ReceiveEvents),
            )
            .add_system(step);
    }
}

fn setup(
    mut commands: Commands,
    window: Res<Window>,
    mut meshes: ResMut<Assets<TriMesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    let width = window.resolution.physical_width();
    let height = window.resolution.physical_height();

    // plane
    commands.spawn(RenderObjectBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: materials.add(Color::from_rgb_f32(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(TriMesh::from(shape::Cube { size: 10.0 })),
            material: materials.add(Color::from_rgb_f32(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 5.0, 0.0),
            ..Default::default()
        })
        .insert(CubeMarker);
    // light
    commands.insert_resource(AmbientLight::new(0.3, Color::RED));
    commands.spawn(PointLight {
        position: Vec3::new(40.0, 80.0, 40.0),
        intensity: 1.0,
        ..Default::default()
    });
    commands.spawn(DirectionalLight {
        direction: Vec3::new(0.0, -1.0, -2.0),
        intensity: 1.0,
        color: Color::BLUE,
    });
    // camera
    commands.spawn(CameraComponent::new(
        Camera::new_orthographic(
            Viewport::new_at_origin(width, height),
            Vec3::new(50.0, 50.0, 50.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            50.0,
            0.0,
            1000.0,
        ),
        // render before the "main pass" camera
        0,
        ClearOperation::default(),
        RenderTarget::Screen,
    ));
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

    let x = degrees_to_radians(*rotation).cos() * 10.0;
    let z = degrees_to_radians(*rotation).sin() * 10.0;

    let mut transform = cube_q.single_mut();

    transform.position.x = x;
    transform.position.z = z;
}

fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}
