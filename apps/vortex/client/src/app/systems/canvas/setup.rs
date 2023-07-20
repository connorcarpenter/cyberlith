use bevy_ecs::system::{Commands, Res, ResMut};
use bevy_log::info;

use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh, CpuTexture2D},
    components::{
        AmbientLight, Camera, CameraBundle, ClearOperation, OrthographicProjection, PointLight,
        Projection, RenderLayers, RenderObjectBundle, RenderTarget, Transform, Viewport,
    },
    Assets, Handle,
};
use render_egui::EguiUserTextures;

use crate::app::{
    components::{HoverCircle, SelectCircle, SelectLine, Vertex2d},
    config::AppConfig,
    resources::canvas_manager::CanvasManager, shapes::create_2d_edge_arrow
};

pub fn setup(
    config: Res<AppConfig>,
    mut commands: Commands,
    mut canvas_manager: ResMut<CanvasManager>,
    mut textures: ResMut<Assets<CpuTexture2D>>,
    mut user_textures: ResMut<EguiUserTextures>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
) {
    info!("Environment: {}", config.general.env_name);

    // Canvas Texture
    let texture_size = Vec2::new(1130.0, 672.0);
    let canvas_texture_handle =
        new_render_texture(&texture_size, &mut textures, &mut user_textures);
    canvas_manager.set_canvas_texture(texture_size, canvas_texture_handle.clone());

    setup_3d_scene(
        &mut commands,
        &mut canvas_manager,
        &texture_size,
        canvas_texture_handle,
    );
    setup_2d_scene(
        &mut commands,
        &mut canvas_manager,
        &mut meshes,
        &mut materials,
        &texture_size,
        canvas_texture_handle,
    );
}

fn setup_2d_scene(
    commands: &mut Commands,
    canvas_manager: &mut CanvasManager,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    texture_size: &Vec2,
    canvas_texture_handle: Handle<CpuTexture2D>,
) {
    canvas_manager.layer_2d = RenderLayers::layer(2);

    // light
    commands
        .spawn(AmbientLight {
            intensity: 1.0,
            color: Color::WHITE,
            ..Default::default()
        })
        .insert(canvas_manager.layer_2d);

    // camera
    let mut camera_bundle = CameraBundle::new_2d(&Viewport::new_at_origin(
        texture_size.x as u32,
        texture_size.y as u32,
    ));
    camera_bundle.camera.target = RenderTarget::Image(canvas_texture_handle);
    camera_bundle.camera.is_active = false;
    camera_bundle.camera.order = 1;
    let camera_entity = commands
        .spawn(camera_bundle)
        .insert(canvas_manager.layer_2d)
        .id();

    canvas_manager.camera_2d = Some(camera_entity);

    // hover circle
    let mut hover_circle_components = RenderObjectBundle::circle(
        meshes,
        materials,
        Vec2::ZERO,
        HoverCircle::DISPLAY_RADIUS,
        Vertex2d::SUBDIVISIONS,
        Color::GREEN,
        Some(1),
    );
    hover_circle_components.visibility.visible = false;
    let hover_circle_entity = commands
        .spawn(hover_circle_components)
        .insert(canvas_manager.layer_2d)
        .insert(HoverCircle)
        .id();
    canvas_manager.hover_circle_entity = Some(hover_circle_entity);

    // select circle
    let mut select_circle_components = RenderObjectBundle::circle(
        meshes,
        materials,
        Vec2::ZERO,
        SelectCircle::RADIUS,
        Vertex2d::SUBDIVISIONS,
        Color::WHITE,
        Some(1),
    );
    select_circle_components.visibility.visible = false;
    let select_circle_entity = commands
        .spawn(select_circle_components)
        .insert(canvas_manager.layer_2d)
        .insert(SelectCircle)
        .id();
    canvas_manager.select_circle_entity = Some(select_circle_entity);

    // select line
    let mut select_line_components = create_2d_edge_arrow(meshes, materials, Vec2::ZERO, Vec2::X, Color::WHITE);
    select_line_components.visibility.visible = false;
    let select_line_entity = commands
        .spawn(select_line_components)
        .insert(canvas_manager.layer_2d)
        .insert(SelectLine)
        .id();
    canvas_manager.select_line_entity = Some(select_line_entity);
}

fn setup_3d_scene(
    commands: &mut Commands,
    canvas_manager: &mut CanvasManager,
    texture_size: &Vec2,
    canvas_texture_handle: Handle<CpuTexture2D>,
) {
    canvas_manager.layer_3d = RenderLayers::layer(3);

    // Ambient Light
    commands
        .spawn(AmbientLight::new(0.01, Color::WHITE))
        .insert(canvas_manager.layer_3d);
    commands
        .spawn(PointLight {
            position: Vec3::new(60.0, 60.0, 90.0),
            color: Color::WHITE,
            intensity: 0.2,
            ..Default::default()
        })
        .insert(canvas_manager.layer_3d);

    // Camera
    let camera_entity = commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: Some(Viewport::new_at_origin(
                    texture_size.x as u32,
                    texture_size.y as u32,
                )),
                order: 0,
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Image(canvas_texture_handle),
                ..Default::default()
            },
            transform: Transform::from_xyz(50.0, 0.0, 0.0) // from front
                .looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Orthographic(OrthographicProjection::new(
                texture_size.y,
                0.0,
                1000.0,
            )),
        })
        .insert(canvas_manager.layer_3d)
        .id();
    canvas_manager.camera_3d = Some(camera_entity);
}

fn new_render_texture(
    texture_size: &Vec2,
    textures: &mut Assets<CpuTexture2D>,
    user_textures: &mut EguiUserTextures,
) -> Handle<CpuTexture2D> {
    // This is the texture that will be rendered to.
    let texture = CpuTexture2D::from_size(texture_size.x as u32, texture_size.y as u32);

    let texture_handle = textures.add(texture);
    user_textures.add_texture(&texture_handle);

    texture_handle
}
