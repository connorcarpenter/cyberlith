use bevy_app::{App, Plugin};
use bevy_asset::{Assets, Handle};
use bevy_core_pipeline::clear_color::ClearColorConfig;
use bevy_core_pipeline::core_3d::{Camera3d, Camera3dBundle};
use bevy_ecs::system::Query;
use bevy_ecs::system::{Commands, ResMut, Resource};
use bevy_log::info;
use bevy_math::Vec3;
use bevy_pbr::{PbrBundle, PbrPlugin, PointLight, PointLightBundle, StandardMaterial};
use bevy_render::camera::{Camera, RenderTarget};
use bevy_render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy_render::{
    color::Color,
    mesh::{shape, Mesh},
    texture::Image,
};
use bevy_transform::components::Transform;
use bevy_window::Window;

#[derive(Resource)]
pub struct GameClientImage(pub Handle<Image>);

pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PbrPlugin::default());
    }
}

pub fn setup(
    mut commands: Commands,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let window = windows.single();

    // image
    let size = Extent3d {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        ..Default::default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..Default::default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);
    commands.insert_resource(GameClientImage(image_handle.clone()));

    info!("inserted image!");

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        camera_3d: Camera3d {
            clear_color: ClearColorConfig::Custom(Color::rgba(0.0, 0.0, 0.0, 1.0)),
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
