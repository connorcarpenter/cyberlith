use bevy_app::{App, Plugin};
use bevy_ecs::{
    component::Component,
    query::With,
    schedule::IntoSystemConfigs,
    system::{Commands, Query, Res, ResMut, Resource},
};
use bevy_log::info;
use naia_bevy_client::{ClientConfig, Plugin as ClientPlugin, ReceiveEvents};

use math::Vec3;
use render_api::{
    Assets,
    base::{Color, CpuMesh, PbrMaterial, Texture2D},
    components::{
        AmbientLight, Camera, CameraBundle, ClearOperation, OrthographicProjection, PointLight,
        Projection, RenderLayers, RenderObjectBundle, RenderTarget, Transform, Viewport,
    },
    Handle, resources::WindowSettings, shapes,
};
use render_egui::EguiUserTextures;
use vortex_proto::{
    components::{EntryKind, FileSystemEntry},
    protocol,
};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    config::{AppConfig, ConfigPlugin},
    events::LoginEvent,
    resources::{action_stack::ActionStack, global::Global},
    systems::network,
    ui,
    ui::{AxesCamerasVisible, UiState},
};
use crate::app::resources::tab_manager::TabManager;

pub struct VortexPlugin;

impl Plugin for VortexPlugin {
    fn build(&self, app: &mut App) {
        // setup Global
        let project_root_entity = app
            .world
            .spawn_empty()
            .insert(FileSystemParent::new())
            .insert(FileSystemUiState::new_root())
            .insert(FileSystemEntry::new("Project", EntryKind::Directory))
            .id();
        let global_resource = Global::new(project_root_entity);

        app
            // Add Config
            .add_plugin(ConfigPlugin)
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Vortex".to_string(),
                max_size: Some((1280, 720)),
                ..Default::default()
            })
            // Networking Plugin
            .add_plugin(ClientPlugin::new(ClientConfig::default(), protocol()))
            .add_event::<LoginEvent>()
            // Networking Systems
            .add_system(network::login)
            .add_systems(
                (
                    network::connect_events,
                    network::disconnect_events,
                    network::reject_events,
                    network::error_events,
                    network::spawn_entity_events,
                    network::despawn_entity_events,
                    network::insert_component_events,
                    network::update_component_events,
                    network::remove_component_events,
                    network::auth_granted_events,
                    network::auth_denied_events,
                    network::auth_reset_events,
                )
                    .chain()
                    .in_set(ReceiveEvents),
            )
            // UI Configuration
            .insert_resource(UiState::new())
            .insert_resource(AxesCamerasVisible(false))
            .insert_resource(global_resource)
            .insert_resource(TabManager::new())
            .insert_resource(ActionStack::new())
            .add_system(ui::main)
            .add_system(ui::sync_axes_cameras_visibility)
            // 3D Configuration
            .add_startup_system(setup)
            .add_system(rotate);
    }
}

// Marks the preview pass cube.
#[derive(Component)]
struct SkeletonCube;

#[derive(Resource)]
pub struct LeftTopTexture(pub Handle<Texture2D>);

#[derive(Resource)]
pub struct LeftBottomTexture(pub Handle<Texture2D>);

#[derive(Resource)]
pub struct RightTopTexture(pub Handle<Texture2D>);

#[derive(Resource)]
pub struct RightBottomTexture(pub Handle<Texture2D>);

fn setup(
    config: Res<AppConfig>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
    mut textures: ResMut<Assets<Texture2D>>,
    mut user_textures: ResMut<EguiUserTextures>,
) {
    info!("Environment: {}", config.general.env_name);

    // This specifies the layer used for the preview pass, which will be attached to the preview pass camera and cube.
    let preview_pass_layer = RenderLayers::layer(1);

    // Cube
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(CpuMesh::from(shapes::Cube { size: 50.0 })),
            material: materials.add(Color::from_rgb_f32(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
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

    // Cameras
    let texture_size = 300;
    let projection = Projection::Orthographic(OrthographicProjection::default());
    let clear_op = ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0);
    let viewport = Some(Viewport::new_at_origin(texture_size, texture_size));
    let center_target = Vec3::new(0.0, 0.0, 0.0);

    // LEFT TOP
    let left_top_texture = new_render_texture(texture_size, &mut textures, &mut user_textures);
    commands.insert_resource(LeftTopTexture(left_top_texture.clone()));

    commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: viewport.clone(),
                order: 0,
                clear_operation: clear_op.clone(),
                target: RenderTarget::Image(left_top_texture),
                ..Default::default()
            },
            transform: Transform::from_xyz(60.0, 0.0, 0.0) // cube facing front?
                .looking_at(center_target, Vec3::Y),
            projection: projection.clone(),
        })
        .insert(preview_pass_layer);

    // LEFT BOTTOM
    let left_bottom_texture = new_render_texture(texture_size, &mut textures, &mut user_textures);
    commands.insert_resource(LeftBottomTexture(left_bottom_texture.clone()));

    commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: viewport.clone(),
                order: 1,
                clear_operation: clear_op.clone(),
                target: RenderTarget::Image(left_bottom_texture),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 60.0) // cube facing right?
                .looking_at(center_target, Vec3::Y),
            projection: projection.clone(),
        })
        .insert(preview_pass_layer);

    // RIGHT TOP
    let right_top_texture = new_render_texture(texture_size, &mut textures, &mut user_textures);
    commands.insert_resource(RightTopTexture(right_top_texture.clone()));

    commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: viewport.clone(),
                order: 2,
                clear_operation: clear_op.clone(),
                target: RenderTarget::Image(right_top_texture),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 60.0, 1.0) // cube facing top? // NOTE: z-value of 1.0 necessary to render anything
                .looking_at(center_target, Vec3::Y),
            projection: projection.clone(),
        })
        .insert(preview_pass_layer);

    // RIGHT BOTTOM
    let right_bottom_texture = new_render_texture(texture_size, &mut textures, &mut user_textures);
    commands.insert_resource(RightBottomTexture(right_bottom_texture.clone()));

    commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: viewport.clone(),
                order: 3,
                clear_operation: clear_op.clone(),
                target: RenderTarget::Image(right_bottom_texture),
                ..Default::default()
            },
            transform: Transform::from_xyz(30.0, 60.0, 30.0).looking_at(center_target, Vec3::Y),
            projection: projection.clone(),
        })
        .insert(preview_pass_layer);
}

fn new_render_texture(
    texture_size: u32,
    textures: &mut Assets<Texture2D>,
    user_textures: &mut EguiUserTextures,
) -> Handle<Texture2D> {
    // This is the texture that will be rendered to.
    let texture = Texture2D::from_size(texture_size, texture_size);

    let texture_handle = textures.add(texture);
    user_textures.add_texture(&texture_handle);

    texture_handle
}

// Rotates the cubes.
fn rotate(mut query: Query<&mut Transform, With<SkeletonCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(0.015);
        transform.rotate_z(0.013);
    }
}
