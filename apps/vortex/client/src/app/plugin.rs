use bevy_app::{App, Plugin};
use bevy_ecs::{
    component::Component,
    query::{Or, With},
    schedule::IntoSystemConfigs,
    system::{Commands, Local, Query, Res, ResMut, Resource},
};

use naia_bevy_client::{
    ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin, ReceiveEvents,
};

use math::{Quat, Vec3};
use render_api::{
    base::{Camera, Color, PbrMaterial, Texture2D, TriMesh, Viewport},
    components::{AmbientLight, CameraBundle, ClearOperation, DirectionalLight, PointLight, RenderLayers, RenderObjectBundle, Transform, RenderTarget},
    resources::WindowSettings,
    shapes, Assets, Handle,
    Window,
};
use render_egui::{
    egui,
    egui::{Modifiers, Ui, Widget},
    EguiContext, EguiUserTextures, GUI,
};

use vortex_proto::protocol;

use crate::app::{
    network, ui,
    ui::{widgets, UiState},
};

pub struct VortexPlugin;

impl Plugin for VortexPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Vortex".to_string(),
                max_size: Some((1280, 720)),
                ..Default::default()
            })
            // Add Naia Client Plugin
            // .add_plugin(NaiaClientPlugin::new(
            //     NaiaClientConfig::default(),
            //     protocol(),
            // ))
            // Startup Systems
            // .add_startup_system(network::init)
            .insert_resource(UiState::default())
            .insert_resource(ProjectTree(widgets::Tree::new()))
            .add_startup_system(setup)
            .add_system(rotate)
            .add_system(ui::main)
        // Receive Client Events
        // .add_systems(
        //     (
        //         network::connect_events,
        //         network::disconnect_events,
        //         network::reject_events,
        //         network::error_events,
        //     )
        //         .chain()
        //         .in_set(ReceiveEvents),
        // )
        // .add_system(step);
        ;
    }
}

#[derive(Resource)]
pub struct ProjectTree(pub widgets::Tree);

// Marks the preview pass cube.
#[derive(Component)]
struct SkeletonCube;

#[derive(Resource)]
pub struct LeftTopTexture(pub Handle<Texture2D>);

fn setup(
    mut commands: Commands,
    window: Res<Window>,
    mut meshes: ResMut<Assets<TriMesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
    mut textures: ResMut<Assets<Texture2D>>,
    mut user_textures: ResMut<EguiUserTextures>,
) {
    // This is the texture that will be rendered to.
    let texture_width = 300;
    let texture_height = 300;
    let mut texture = Texture2D::from_size(texture_width, texture_height);

    let texture_handle = textures.add(texture);
    user_textures.add_texture(&texture_handle);
    commands.insert_resource(LeftTopTexture(texture_handle.clone()));

    // This specifies the layer used for the preview pass, which will be attached to the preview pass camera and cube.
    let preview_pass_layer = RenderLayers::layer(1);

    // Cube
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(TriMesh::from(shapes::Cube { size: 20.0 })),
            material: materials.add(Color::from_rgb_f32(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 10.0, 0.0),
        })
        .insert(SkeletonCube)
        .insert(preview_pass_layer);

    // Ambient Light
    commands
        .spawn(AmbientLight::new(0.01, Color::WHITE))
        .insert(preview_pass_layer);
    commands
        .spawn(PointLight {
            position: Vec3::new(50.0, 150.0, 100.0),
            color: Color::WHITE,
            intensity: 0.1,
            ..Default::default()
        })
        .insert(preview_pass_layer);

    // Camera
    commands
        .spawn(CameraBundle {
            camera: Camera::new_orthographic(
                Viewport::new_at_origin(texture_width, texture_height),
                50.0,
                0.0,
                1000.0,
                0,
                ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                RenderTarget::Image(texture_handle),
            ),
            transform: Transform::from_xyz(10.0, 20.0, 10.0)
                .looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        })
        .insert(preview_pass_layer);
}

// Rotates the cubes.
fn rotate(mut query: Query<&mut Transform, With<SkeletonCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(0.015);
        transform.rotate_z(0.013);
    }
}
