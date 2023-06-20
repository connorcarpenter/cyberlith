use bevy_app::{App, Plugin};
use bevy_ecs::system::{Commands, Res, ResMut};

use render_api::{
    Assets,
    base::{Color, CpuMaterial, CpuMesh},
    components::{AmbientLight, CameraBundle, RenderObjectBundle},
    resources::WindowSettings, Window,
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
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
) {
    // circle

    commands.spawn(RenderObjectBundle::circle(
        &mut meshes,
        &mut materials,
        640.0,
        360.0,
        50.0,
        20,
        Color::GREEN,
    ));
    commands.spawn(RenderObjectBundle::circle(
        &mut meshes,
        &mut materials,
        480.0,
        240.0,
        20.0,
        20,
        Color::GREEN,
    ));
    commands.spawn(RenderObjectBundle::circle(
        &mut meshes,
        &mut materials,
        560.0,
        480.0,
        30.0,
        20,
        Color::GREEN,
    ));

    // light
    commands.spawn(AmbientLight {
        intensity: 1.0,
        color: Color::WHITE,
        ..Default::default()
    });
    // camera
    commands.spawn(CameraBundle::new_2d(&window.viewport()));
}

fn step() {}
