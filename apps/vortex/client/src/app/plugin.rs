use bevy_app::{App, Plugin};
use bevy_ecs::{
    component::Component,
    query::With,
    schedule::IntoSystemConfigs,
    system::{Commands, Query, Res, ResMut, Resource},
};
use bevy_ecs::query::Without;
use bevy_log::info;
use naia_bevy_client::{ClientConfig, Plugin as ClientPlugin, ReceiveEvents};

use math::{Mat4, Vec3};
use render_api::{
    Assets,
    base::{Color, CpuMaterial, CpuMesh, CpuTexture2D},
    components::{
        AmbientLight, Camera, CameraBundle, ClearOperation, OrthographicProjection, PointLight,
        Projection, RenderLayers, RenderObjectBundle, RenderTarget, Transform, Viewport,
    },
    Handle, resources::WindowSettings, shapes,
};
use render_api::components::CameraProjection;
use render_egui::EguiUserTextures;
use vortex_proto::{
    components::{EntryKind, FileSystemEntry},
    protocol,
};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    config::{AppConfig, ConfigPlugin},
    events::LoginEvent,
    resources::{action_stack::ActionStack, global::Global, tab_manager::TabManager},
    systems::network,
    ui,
    ui::{AxesCamerasVisible, UiState},
};

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
            .add_system(step);
    }
}

// Marks the preview pass cube.
#[derive(Component)]
struct SkeletonCube;

#[derive(Resource)]
pub struct CanvasTexture(pub Handle<CpuTexture2D>);

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

    // Canvas Texture
    let texture_size = 300;
    let canvas_texture_handle = new_render_texture(texture_size, &mut textures, &mut user_textures);
    commands.insert_resource(CanvasTexture(canvas_texture_handle.clone()));

    setup_3d_scene(&mut commands, &mut global, &mut meshes, &mut materials, texture_size, canvas_texture_handle);
    setup_2d_scene(&mut commands, &mut global, texture_size, canvas_texture_handle);
}

fn setup_2d_scene(
    commands: &mut Commands,
    global: &mut Global,
    texture_size: u32,
    canvas_texture_handle: Handle<CpuTexture2D>,
) {
    global.layer_2d = RenderLayers::layer(2);

    // light
    commands.spawn(AmbientLight {
        intensity: 1.0,
        color: Color::WHITE,
        ..Default::default()
    }).insert(global.layer_2d);

    // camera
    let mut camera_bundle = CameraBundle::new_2d(&Viewport::new_at_origin(texture_size, texture_size));
    camera_bundle.camera.target = RenderTarget::Image(canvas_texture_handle);
    camera_bundle.camera.is_active = false;
    camera_bundle.camera.order = 1;
    let camera_entity = commands.spawn(camera_bundle).insert(global.layer_2d).id();

    global.camera_2d = Some(camera_entity);
}

fn setup_3d_scene(
    commands: &mut Commands,
    global: &mut Global,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    texture_size: u32,
    canvas_texture_handle: Handle<CpuTexture2D>,
) {
    global.layer_3d = RenderLayers::layer(3);

    // Cube
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(shapes::Cube),
            material: materials.add(Color::from_rgb_f32(0.8, 0.7, 0.6)),
            transform: Transform::IDENTITY.with_scale(Vec3::splat(10.0)),
        })
        .insert(SkeletonCube)
        .insert(global.layer_3d);

    // Ambient Light
    commands
        .spawn(AmbientLight::new(0.01, Color::WHITE))
        .insert(global.layer_3d);
    commands
        .spawn(PointLight {
            position: Vec3::new(50.0, 150.0, 100.0),
            color: Color::WHITE,
            intensity: 0.1,
            ..Default::default()
        })
        .insert(global.layer_3d);

    // Camera
    let camera_entity = commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: Some(Viewport::new_at_origin(texture_size, texture_size)),
                order: 0,
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Image(canvas_texture_handle),
                ..Default::default()
            },
            transform: Transform::from_xyz(60.0, 30.0, 60.0) // isometric-ish
                .looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Orthographic(OrthographicProjection::default()),
        })
        .insert(global.layer_3d)
        .id();
    global.camera_3d = Some(camera_entity);
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

fn step(
    mut commands: Commands,
    mut global: ResMut<Global>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut cube_query: Query<(&Handle<CpuMesh>, &mut Transform), With<SkeletonCube>>,
    mut transform_query: Query<&mut Transform, (Without<SkeletonCube>, With<Handle<CpuMesh>>)>,
    camera_query: Query<(&Camera, &Transform, &Projection), Without<Handle<CpuMesh>>>,
) {
    let mut cube: Option<(Handle<CpuMesh>, Transform)> = None;

    // Rotates the cube
    for (mesh_handle, mut transform) in &mut cube_query {
        transform.rotate_x(0.015);
        transform.rotate_z(0.013);
        if cube.is_some() {
            panic!("should only be one cube..");
        }
        cube = Some((mesh_handle.clone(), transform.clone()));
    }

    // Get mesh
    let Some((mesh_handle, transform)) = cube else {
        panic!("no cube found");
    };

    let mesh = meshes.get(&mesh_handle).unwrap();
    let mut rotated_mesh = mesh.clone();

    // get number of vertices
    let mesh_vertex_count = mesh.vertex_count();
    let loaded_vertex_count = global.vertices_2d.len();

    if loaded_vertex_count != mesh_vertex_count {
        for loaded_vertex in global.vertices_2d.iter() {
            commands.entity(*loaded_vertex).despawn();
        }

        global.vertices_2d.clear();

        for i in 0..mesh_vertex_count {
            info!("spawning vertex: {:?}", i);
            let vertex_entity = commands.spawn(RenderObjectBundle::circle(
                &mut meshes,
                &mut materials,
                0.0,
                0.0,
                4.0,
                12,
                Color::GREEN,
                false,
            )).insert(global.layer_2d).id();

            global.vertices_2d.push(vertex_entity);
        }
    }

    // rotate mesh
    rotated_mesh.transform(&transform.compute_matrix());

    // update positions
    if global.camera_2d.is_none() || global.camera_3d.is_none() {
        return;
    }
    let (camera_2d, _, _) = camera_query.get(global.camera_2d.unwrap()).unwrap();
    let (camera_3d, camera_3d_transform, camera_3d_proj) = camera_query.get(global.camera_3d.unwrap()).unwrap();
    let camera_2d_viewport = camera_2d.viewport.unwrap();
    let camera_3d_viewport = camera_3d.viewport.unwrap();
    let camera_3d_proj_matrix = camera_3d_proj.projection_matrix(&camera_3d_viewport);

    for (index, pos) in rotated_mesh.positions.0.iter().enumerate() {
        let vertex_entity = global.vertices_2d[index];

        if let Ok(mut vertex_transform) = transform_query.get_mut(vertex_entity) {

            // todo: change this to convert from 3d to 2d
            let point_2d = convert_3d_to_2d(
                &camera_3d_transform.view_matrix(),
                &camera_3d_proj_matrix,
                &camera_2d_viewport,
                pos,
            );
            vertex_transform.translation.x = point_2d.x;
            vertex_transform.translation.y = point_2d.y;
        }
    }
}

fn convert_3d_to_2d(
    view_matrix: &Mat4,
    projection_matrix: &Mat4,
    viewport: &Viewport,
    point_3d: &Vec3,
) -> Vec3 {
    let viewport_width = viewport.width as f32;
    let viewport_height = viewport.height as f32;

    // Calculate the clip space coordinate
    let clip_space_coordinate = *projection_matrix * *view_matrix * point_3d.extend(1.0);

    // Normalize the clip space coordinate
    let clip_space_vec3 = Vec3::new(clip_space_coordinate.x, clip_space_coordinate.y, clip_space_coordinate.z);
    let normalized_device_coordinate = clip_space_vec3 / clip_space_coordinate.w;

    // Convert the normalized device coordinate to screen space coordinate
    let screen_space_x = (normalized_device_coordinate.x + 1.0) * 0.5 * viewport_width;
    let screen_space_y = (1.0 - normalized_device_coordinate.y) * 0.5 * viewport_height;
    let screen_space_d = normalized_device_coordinate.z; // -1.0 -> 1.0 (near -> far)

    // The resulting screen space coordinates
    Vec3::new(screen_space_x, screen_space_y, screen_space_d)
}