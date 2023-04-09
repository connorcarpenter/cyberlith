use bevy_app::{App, Plugin};
use bevy_ecs::{
    component::Component,
    query::{With, Or},
    schedule::IntoSystemConfigs,
    system::{Commands, Local, Query, Res, ResMut, Resource},
};

use naia_bevy_client::{
    ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin, ReceiveEvents,
};

use math::{Quat, Vec3};
use render_api::{
    base::{Camera, Color, PbrMaterial, Texture2D, TriMesh, Viewport},
    shapes, AmbientLight, Assets, CameraComponent, ClearOperation, DirectionalLight, Handle,
    PointLight, RenderLayers, RenderObjectBundle, RenderTarget, Transform, Window,
};
use render_egui::{egui, EguiContext, EguiUserTextures, GUI, egui::{Modifiers, Ui, Widget}};

use editor_proto::protocol;

use crate::app::{network, ui};
use crate::app::ui::UiState;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add Naia Client Plugin
            // .add_plugin(NaiaClientPlugin::new(
            //     NaiaClientConfig::default(),
            //     protocol(),
            // ))
            // Startup Systems
            // .add_startup_system(network::init)
            .insert_resource(UiState::default())
            .add_startup_system(setup)
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

// Marks the preview pass cube.
#[derive(Component)]
struct PreviewPassCube;

#[derive(Resource)]
struct CubePreviewImage(Handle<Texture2D>);

fn setup(
    mut commands: Commands,
    window: Res<Window>,
    mut images: ResMut<Assets<Texture2D>>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
    mut meshes: ResMut<Assets<TriMesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
) {
    // This is the texture that will be rendered to.
    let texture_width = 512;
    let texture_height = 512;
    let mut texture = Texture2D::from_size(texture_width, texture_height);

    let texture_handle = images.add(texture);
    egui_user_textures.add_texture(&texture_handle);
    commands.insert_resource(CubePreviewImage(texture_handle.clone()));

    // This specifies the layer used for the preview pass, which will be attached to the preview pass camera and cube.
    let preview_pass_layer = RenderLayers::layer(1);

    // Cube
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(TriMesh::from(shapes::Cube { size: 5.0 })),
            material: materials.add(Color::from_rgb_f32(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
        })
        .insert(PreviewPassCube)
        .insert(preview_pass_layer);

    // Ambient Light
    commands.insert_resource(AmbientLight::new(1.0, Color::WHITE));

    // Camera
    commands.spawn(CameraComponent::new(
        Camera::new_orthographic(
            window.viewport(),
            Vec3::new(50.0, 50.0, 50.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            50.0,
            0.0,
            1000.0,
        ),
        0,
        ClearOperation::default(),
        RenderTarget::Image(texture_handle),
    ))
        .insert(preview_pass_layer);
}

// Rotates the cubes.
fn rotator_system(
    mut query: Query<&mut Transform, With<PreviewPassCube>>,
) {
    for mut transform in &mut query {
        transform.rotate_x(0.015);
        transform.rotate_z(0.013);
    }
}