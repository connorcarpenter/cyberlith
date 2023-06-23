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

    commands.spawn(RenderObjectBundle::rectangle(
        &mut meshes,
        &mut materials,
        480.0,
        240.0,
        4.0,
        4.0,
        Color::GREEN,
        false,
    ));
    commands.spawn(RenderObjectBundle::rectangle(
        &mut meshes,
        &mut materials,
        480.0,
        240.0,
        7.5,
        7.5,
        Color::GREEN,
        true,
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
