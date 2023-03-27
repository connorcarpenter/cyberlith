use bevy_app::{App, Plugin};
use bevy_ecs::system::{Commands, Res, ResMut, Resource};
use bevy_log::info;
use bevy_render::{
    math::Vec3, shape, Assets, Camera, Camera3d, Camera3dBundle, ClearColorConfig, Color, Handle,
    Image, Mesh, OrthographicProjection, PointLight, PointLightBundle, RenderObjectBundle,
    RenderTarget, StandardMaterial, Transform, Window,
};

#[derive(Resource)]
pub struct GameClientImage(pub Handle<Image>);

pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, _app: &mut App) {}
}

pub fn setup(
    mut commands: Commands,
    window: Res<Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // This is the texture that will be rendered to.
    let image = Image::new(
        window.resolution.physical_width(),
        window.resolution.physical_height(),
    );

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
        projection: OrthographicProjection {
            scale: 3.0,
            ..Default::default()
        }
        .into(),
        transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
