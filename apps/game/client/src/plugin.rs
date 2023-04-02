use bevy_app::{App, Plugin};
use bevy_ecs::query::With;
use bevy_ecs::{
    component::Component,
    system::{Commands, Local, Query, Res, ResMut, Resource},
};
use bevy_log::info;
use render_api::{
    base::{Camera, Color, PbrMaterial, Texture2D, TriMesh, Vec3, Viewport},
    shape, Assets, CameraComponent, ClearOperation, Handle, PointLight, RenderObjectBundle,
    RenderTarget, Transform, Window,
};

#[derive(Component)]
pub struct CubeMarker;

#[derive(Resource)]
pub struct GameClientTexture(pub Handle<Texture2D>);

pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(step);
    }
}

pub fn setup(
    mut commands: Commands,
    window: Res<Window>,
    mut meshes: ResMut<Assets<TriMesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
    mut textures: ResMut<Assets<Texture2D>>,
) {
    let width = window.resolution.physical_width();
    let height = window.resolution.physical_height();

    // This is the texture that will be rendered to.
    let texture = Texture2D::from_size(width, height);

    let texture_handle = textures.add(texture);
    commands.insert_resource(GameClientTexture(texture_handle.clone()));

    info!("inserted image!");

    // plane
    commands.spawn(RenderObjectBundle {
        mesh: meshes.add(shape::Plane::from_size(50.0).into()),
        material: materials.add(Color::from_rgb_f32(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    // cube
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(TriMesh::from(shape::Cube { size: 10.0 })),
            material: materials.add(Color::from_rgb_f32(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 5.0, 0.0),
            ..Default::default()
        })
        .insert(CubeMarker);
    // light
    commands.spawn(PointLight {
        position: Vec3::new(40.0, 80.0, 40.0),
        intensity: 1.0,
        ..Default::default()
    });
    // camera
    commands.spawn(CameraComponent::new(
        Camera::new_orthographic(
            Viewport::new_at_origin(width, height),
            Vec3::new(50.0, 50.0, 50.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            50.0,
            0.0,
            1000.0,
        ),
        // render before the "main pass" camera
        0,
        ClearOperation::default(),
        RenderTarget::Image(texture_handle),
    ));
}

fn step(mut cube_q: Query<&mut Transform, With<CubeMarker>>, mut rotation: Local<f32>) {
    if *rotation == 0.0 {
        *rotation = 0.01;
    } else {
        *rotation += 1.0;
        if *rotation > 359.0 {
            *rotation = 0.01;
        }
    }

    let x = degrees_to_radians(*rotation).cos() * 10.0;
    let z = degrees_to_radians(*rotation).sin() * 10.0;

    let mut transform = cube_q.single_mut();

    transform.position.x = x;
    transform.position.z = z;
}

fn degrees_to_radians(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}
