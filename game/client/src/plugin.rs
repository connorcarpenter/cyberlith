use bevy_app::{App, Plugin};
use bevy_ecs::system::{Query, Commands, ResMut, Res, Resource};
use bevy_log::info;
use bevy_render::{shape, math::Vec3, Handle, Assets, Image, Window, Mesh, ClearColorConfig, RenderTarget, StandardMaterial, RenderObjectBundle, PointLightBundle, PointLight, Camera3dBundle, Camera, Camera3d, Color, Transform};

#[derive(Resource)]
pub struct GameClientImage(pub Handle<Image>);

pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, _app: &mut App) {

    }
}

pub fn setup(
    mut commands: Commands,
    window: Res<Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // This is the texture that will be rendered to.
    let mut image = Image::new(window.resolution.physical_width(), window.resolution.physical_height());

    let image_handle = images.add(image);
    commands.insert_resource(GameClientImage(image_handle.clone()));

    info!("inserted image!");

    // plane
    commands.spawn(RenderObjectBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands.spawn(RenderObjectBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        camera_3d: Camera3d {
            clear_color: ClearColorConfig::Custom(Color::rgb(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        camera: Camera {
            // render before the "main pass" camera
            order: 0,
            target: RenderTarget::Image(image_handle),
            ..Default::default()
        },
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
