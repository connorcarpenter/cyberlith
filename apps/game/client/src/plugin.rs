use bevy_app::{App, Plugin};
use bevy_ecs::system::{Commands, Res, ResMut, Resource};
use bevy_log::info;
use render_api::{
    base::{Camera, Color, PbrMaterial, Texture2D, TriMesh, Vec3, Viewport},
    shape, Assets, CameraComponent, ClearOperation, Handle, PointLight, PointLightBundle,
    RenderObjectBundle, RenderTarget, Transform, Window,
};

#[derive(Resource)]
pub struct GameClientTexture(pub Handle<Texture2D>);

pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, _app: &mut App) {}
}

pub fn setup(
    mut commands: Commands,
    window: Res<Window>,
    mut meshes: ResMut<Assets<TriMesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
    mut images: ResMut<Assets<Texture2D>>,
) {
    let width = window.resolution.physical_width();
    let height = window.resolution.physical_height();

    // This is the texture that will be rendered to.
    let texture = Texture2D::from_size(width, height);

    let texture_handle = images.add(texture);
    commands.insert_resource(GameClientTexture(texture_handle.clone()));

    info!("inserted image!");

    // plane
    commands.spawn(RenderObjectBundle {
        mesh: meshes.add(shape::Plane::from_size(500.0).into()),
        material: materials.add(Color::from_rgb_f32(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands.spawn(RenderObjectBundle {
        mesh: meshes.add(TriMesh::from(shape::Cube { size: 100.0 })),
        material: materials.add(Color::from_rgb_f32(0.8, 0.7, 0.6).into()),
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
    commands.spawn(
        (CameraComponent::new(
            Camera::new_orthographic(
                Viewport::new_at_origin(width, height),
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                height as f32,
                0.0,
                10.0,
            ),
            // render before the "main pass" camera
            0,
            ClearOperation::default(),
            RenderTarget::Image(texture_handle),
        )),
    );
}
