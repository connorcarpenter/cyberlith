use bevy_app::{App, Plugin};
use bevy_ecs::{
    component::Component,
    query::With,
    schedule::IntoSystemConfigs,
    system::{Commands, Query, Res, ResMut, Resource},
};
use bevy_log::info;
use naia_bevy_client::{ClientConfig, Plugin as ClientPlugin, ReceiveEvents};

use input::{Input, MouseButton};
use math::Vec3;
use render_api::{
    Assets,
    base::{Color, CpuMaterial, CpuMesh, CpuTexture2D},
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
            .insert_resource(AxesCamerasVisible(true))
            .insert_resource(global_resource)
            .insert_resource(TabManager::new())
            .insert_resource(ActionStack::new())
            .add_system(ui::main)
            .add_system(ui::sync_axes_cameras_visibility)
            // 3D Configuration
            .add_startup_system(setup)
            .add_system(step_2d);
    }
}

// Marks the preview pass cube.
#[derive(Component)]
struct SkeletonCube;

#[derive(Resource)]
pub struct WorkspaceTexture(pub Handle<CpuTexture2D>);

fn setup(
    config: Res<AppConfig>,
    mut commands: Commands,
    mut global: ResMut<Global>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut textures: ResMut<Assets<CpuTexture2D>>,
    mut user_textures: ResMut<EguiUserTextures>,
) {
    info!("Environment: {}", config.general.env_name);

    // Workspace Texture
    let texture_size = 300;
    let workspace_texture_handle = new_render_texture(texture_size, &mut textures, &mut user_textures);
    commands.insert_resource(WorkspaceTexture(workspace_texture_handle.clone()));

    //setup_3d_scene(&mut commands, &mut global, &mut meshes, &mut materials, texture_size, workspace_texture_handle);
    setup_2d_scene(&mut commands, &mut global, &mut meshes, &mut materials, texture_size, workspace_texture_handle);
}

fn setup_2d_scene(
    commands: &mut Commands,
    global: &mut Global,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    texture_size: u32,
    workspace_texture_handle: Handle<CpuTexture2D>,
) {
    // circle

    let solid_circle = commands.spawn(RenderObjectBundle::circle(
        meshes,
        materials,
        150.0,
        150.0,
        4.0,
        12,
        Color::GREEN,
        false,
    )).id();
    global.solid_circle = Some(solid_circle);

    let hollow_circle = commands.spawn(RenderObjectBundle::circle(
        meshes,
        materials,
        150.0,
        150.0,
        7.5,
        12,
        Color::GREEN,
        true,
    )).id();
    global.hollow_circle = Some(hollow_circle);

    // light
    commands.spawn(AmbientLight {
        intensity: 1.0,
        color: Color::WHITE,
        ..Default::default()
    });

    // camera
    let mut camera_bundle = CameraBundle::new_2d(&Viewport::new_at_origin(texture_size, texture_size));
    camera_bundle.camera.target = RenderTarget::Image(workspace_texture_handle);
    let camera_entity = commands.spawn(camera_bundle).id();

    global.workspace_camera = Some(camera_entity);
}

fn setup_3d_scene(
    commands: &mut Commands,
    global: &mut Global,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    texture_size: u32,
    workspace_texture_handle: Handle<CpuTexture2D>,
) {
    // This specifies the layer used for the preview pass, which will be attached to the preview pass camera and cube.
    let preview_pass_layer = RenderLayers::layer(1);

    // Cube
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(shapes::Cube),
            material: materials.add(Color::from_rgb_f32(0.8, 0.7, 0.6)),
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

    // Camera
    let camera_entity = commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: Some(Viewport::new_at_origin(texture_size, texture_size)),
                order: 0,
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Image(workspace_texture_handle),
                ..Default::default()
            },
            transform: Transform::from_xyz(60.0, 0.0, 0.0) // cube facing front?
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            projection: Projection::Orthographic(OrthographicProjection::default()),
        })
        .insert(preview_pass_layer)
        .id();
    global.workspace_camera = Some(camera_entity);
}

fn new_render_texture(
    texture_size: u32,
    textures: &mut Assets<CpuTexture2D>,
    user_textures: &mut EguiUserTextures,
) -> Handle<CpuTexture2D> {
    // This is the texture that will be rendered to.
    let texture = CpuTexture2D::from_size(texture_size, texture_size);

    let texture_handle = textures.add(texture);
    user_textures.add_texture(&texture_handle);

    texture_handle
}

fn step_3d(mut query: Query<&mut Transform, With<SkeletonCube>>) {
    // Rotates the cubes.
    for mut transform in &mut query {
        transform.rotate_x(0.015);
        transform.rotate_z(0.013);
    }
}

fn step_2d(
    global: Res<Global>,
    mut query: Query<&mut Transform>,
    input: Res<Input>,
) {
    let mouse_coords = input.mouse();
    if input.is_pressed(MouseButton::Left) {
        if let Some(hollow_circle_id) = global.hollow_circle {
            if let Ok(mut transform) = query.get_mut(hollow_circle_id) {
                transform.translation.x = mouse_coords.x;
                transform.translation.y = mouse_coords.y;
            }
        }
    }

    if let Some(solid_circle_id) = global.solid_circle {
        if let Ok(mut transform) = query.get_mut(solid_circle_id) {
            transform.translation.x = mouse_coords.x;
            transform.translation.y = mouse_coords.y;
        }
    }
}