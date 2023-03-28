use bevy_app::{App, Plugin};
use bevy_ecs::system::{Commands, Res, ResMut, Resource};
use bevy_log::info;
use render_api::{
    shape, Assets, Camera, ClearColorConfig, Color, Handle,
    Image, Mesh, PointLight, PointLightBundle, RenderObjectBundle,
    RenderTarget, StandardMaterial, Transform, Window, InnerCamera, ClearState, Viewport, vec3
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
    let width = window.resolution.physical_width();
    let height = window.resolution.physical_height();

    // This is the texture that will be rendered to.
    let image = Image::new(
        width, height,
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
    commands.spawn((
        Camera::new(
            // render before the "main pass" camera
            0,
            ClearState::default(),
            RenderTarget::Image(image_handle),
            InnerCamera::new_orthographic(
                Viewport::new_at_origo(width, height),
                vec3(5.0, 5.0, 5.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 1.0, 0.0),
                height as f32,
                0.0,
                1000.0,
            ),
        )));
}
