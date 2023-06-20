use bevy_app::{App, Plugin};
use bevy_ecs::system::{Commands, Res, ResMut};

use math::Vec2;
use render_api::{
    Assets,
    base::{Color, PbrMaterial, TriMesh},
    components::{AmbientLight, CameraBundle, RenderObjectBundle, Transform},
    resources::WindowSettings, shapes, Window,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Temp2D".to_string(),
                max_size: Some((1280, 720)),
                ..Default::default()
            })
            // Startup Systems
            .add_startup_system(setup)
            .add_system(step);
    }
}

fn setup(
    mut commands: Commands,
    window: Res<Window>,
    mut meshes: ResMut<Assets<TriMesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    // circle

    //commands.spawn(MaterialMesh2dBundle {
    //         mesh: meshes.add(shape::Circle::new(50.).into()).into(),
    //         material: materials.add(ColorMaterial::from(Color::PURPLE)),
    //         transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
    //         ..default()
    //     });

    commands.spawn(RenderObjectBundle {
        mesh: meshes.add(
            shapes::Triangle::new_2d(
                Vec2::new(0.0, 0.0),
                Vec2::new(0.0, 100.0),
                Vec2::new(100.0, 0.0),
            )
                .into(),
        ),
        material: materials.add(Color::GREEN.into()),
        transform: Transform::from_xy(640.0, 360.0),
        ..Default::default()
    });
    // light
    commands.spawn(AmbientLight {
        intensity: 1.0,
        color: Color::WHITE,
        ..Default::default()
    });
    // commands.spawn(PointLight {
    //     position: Vec3::new(40.0, 80.0, 40.0),
    //     intensity: 0.3,
    //     color: Color::RED,
    //     ..Default::default()
    // });
    // commands.spawn(DirectionalLight {
    //     direction: Vec3::new(0.0, -1.0, -2.0),
    //     intensity: 1.0,
    //     color: Color::WHITE,
    // });
    // camera
    commands.spawn(CameraBundle::new_2d(&window.viewport()));
}

fn step() {}
