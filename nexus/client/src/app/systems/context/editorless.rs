use bevy_app::{App, Plugin};
use bevy_asset::Assets;
use bevy_core_pipeline::core_2d::Camera2dBundle;
use bevy_ecs::prelude::Res;
use bevy_ecs::system::{Commands, Query, ResMut};
use bevy_math::{Vec2, Vec3};
use render_api::camera::Camera;
use render_api::mesh::{shape, Mesh};
use render_api::view::RenderLayers;
use bevy_sprite::{ColorMaterial, MaterialMesh2dBundle};
use bevy_transform::components::Transform;
use bevy_window::Window;

use cybl_game_client::GameClientImage;

pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(step);
    }
}

pub fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_client_image: Res<GameClientImage>,
) {
    // This assumes we only have a single window
    let window = windows.single();
    let size = Vec2::new(
        window.resolution.physical_width() as f32,
        window.resolution.physical_height() as f32,
    );

    // quad mesh handle
    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(size)));

    // material handle
    let material_handle = materials.add(ColorMaterial::from(game_client_image.0.clone()));

    // render layer
    let final_render_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

    // quad entity
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..Default::default()
            },
            ..Default::default()
        },
        final_render_layer,
    ));

    // camera entity
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                // renders after the first main camera which has default value: 0.
                order: 1,
                ..Default::default()
            },
            ..Camera2dBundle::default()
        },
        final_render_layer,
    ));
}

fn step(_game_client_image: Res<GameClientImage>) {}
