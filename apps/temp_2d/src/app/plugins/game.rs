use bevy_app::{App, Plugin};
use bevy_ecs::system::{Commands, Query, Res, ResMut};

use input::Input;
use render_api::{
    Assets,
    base::{Color, CpuMaterial, CpuMesh},
    components::{AmbientLight, CameraBundle, RenderObjectBundle, Transform},
    resources::WindowSettings, Window,
};

use crate::app::resources::Global;

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
            // Add Global Resource
            .insert_resource(Global::new())
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
    mut global: ResMut<Global>,
) {
    // circle

    let solid_circle = commands.spawn(RenderObjectBundle::circle(
        &mut meshes,
        &mut materials,
        480.0,
        240.0,
        4.0,
        12,
        Color::GREEN,
        false,
    )).id();
    global.solid_circle = Some(solid_circle);

    let hollow_circle = commands.spawn(RenderObjectBundle::circle(
        &mut meshes,
        &mut materials,
        480.0,
        240.0,
        7.5,
        12,
        Color::GREEN,
        true,
    )).id();
    global.hollow_circle = Some(hollow_circle);

    // light
    commands.spawn(AmbientLight {
        intensity: 1.0,
        color: Color::WHITE,
        ..Default::default()
    });
    // camera
    commands.spawn(CameraBundle::new_2d(&window.viewport()));
}

fn step(
    global: Res<Global>,
    mut query: Query<&mut Transform>,
    input: Res<Input>,
) {
    // if let Some(hollow_circle_id) = global.hollow_circle {
    //     if let Ok(mut transform) = query.get_mut(hollow_circle_id) {
    //         transform.translation.x = input.mouse_x();
    //         transform.translation.y = input.mouse_y();
    //     }
    // }

    if let Some(solid_circle_id) = global.solid_circle {
        if let Ok(mut transform) = query.get_mut(solid_circle_id) {
            transform.translation.x = input.mouse().x;
            transform.translation.y = input.mouse().y;
        }
    }
}
