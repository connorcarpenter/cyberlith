use bevy_ecs::system::{Commands, Res, ResMut};
use logging::info;

use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh, CpuTexture2D},
    components::{
        AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
        OrthographicProjection, Projection, RenderLayers, RenderObjectBundle, RenderTarget,
        Transform, Viewport,
    },
};
use render_egui::EguiUserTextures;
use storage::{Handle, Storage};

use crate::app::{
    components::{DefaultDraw, SelectCircle, SelectLine, SelectTriangle, Vertex2d},
    config::AppConfig,
    resources::{
        camera_manager::CameraManager, canvas::Canvas, compass::Compass, edge_manager::EdgeManager,
        grid::Grid, icon_manager::IconManager, input::InputManager, vertex_manager::VertexManager,
    },
    shapes::create_2d_edge_line,
};

pub fn setup(
    config: Res<AppConfig>,
    mut commands: Commands,
    mut canvas: ResMut<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut camera_manager: ResMut<CameraManager>,
    mut input_manager: ResMut<InputManager>,
    mut icon_manager: ResMut<IconManager>,
    mut compass: ResMut<Compass>,
    mut grid: ResMut<Grid>,
    mut textures: ResMut<Storage<CpuTexture2D>>,
    mut user_textures: ResMut<EguiUserTextures>,
    mut meshes: ResMut<Storage<CpuMesh>>,
    mut materials: ResMut<Storage<CpuMaterial>>,
) {
    info!("Environment: {}", config.general.env_name);

    // Canvas Texture
    let texture_size = Vec2::new(1130.0, 672.0);
    let canvas_texture_handle =
        new_render_texture(&texture_size, &mut textures, &mut user_textures);
    canvas.set_texture(texture_size, canvas_texture_handle.clone());

    vertex_manager.setup(&mut materials);

    setup_3d_scene(
        &mut commands,
        &mut camera_manager,
        &texture_size,
        canvas_texture_handle,
    );
    setup_2d_scene(
        &mut commands,
        &mut camera_manager,
        &mut input_manager,
        &mut meshes,
        &mut materials,
        &texture_size,
        canvas_texture_handle,
    );
    icon_manager.setup_scene(
        &mut commands,
        &mut meshes,
        &mut materials,
        &texture_size,
        canvas_texture_handle,
    );
    compass.setup_compass(
        &mut commands,
        &mut camera_manager,
        &mut vertex_manager,
        &mut edge_manager,
        &mut meshes,
        &mut materials,
    );
    grid.setup_grid(
        &mut commands,
        &mut camera_manager,
        &mut vertex_manager,
        &mut edge_manager,
        &mut meshes,
        &mut materials,
    );
}

fn setup_2d_scene(
    commands: &mut Commands,
    camera_manager: &mut CameraManager,
    input_manager: &mut InputManager,
    meshes: &mut Storage<CpuMesh>,
    materials: &mut Storage<CpuMaterial>,
    texture_size: &Vec2,
    canvas_texture_handle: Handle<CpuTexture2D>,
) {
    camera_manager.layer_2d = RenderLayers::layer(2);

    let mat_handle_white = materials.add(Color::WHITE);

    // light
    {
        commands
            .spawn(AmbientLight::new(1.0, Color::WHITE))
            .insert(camera_manager.layer_2d);
    }

    // camera
    {
        let mut camera_bundle = CameraBundle::new_2d(&Viewport::new_at_origin(
            texture_size.x as u32,
            texture_size.y as u32,
        ));
        camera_bundle.camera.target = RenderTarget::Image(canvas_texture_handle);
        camera_bundle.camera.is_active = false;
        let camera_entity = commands
            .spawn(camera_bundle)
            .insert(camera_manager.layer_2d)
            .id();

        camera_manager.camera_2d = Some(camera_entity);
    }

    // select circle
    {
        let mut select_circle_components = RenderObjectBundle::circle(
            meshes,
            &mat_handle_white,
            Vec2::ZERO,
            SelectCircle::RADIUS,
            Vertex2d::SUBDIVISIONS,
            Some(1),
        );
        select_circle_components.visibility.visible = false;
        let select_circle_entity = commands
            .spawn(select_circle_components)
            .insert(camera_manager.layer_2d)
            .insert(SelectCircle)
            .insert(DefaultDraw)
            .id();
        input_manager.select_circle_entity = Some(select_circle_entity);
    }

    // select triangle
    {
        let mut select_triangle_components = RenderObjectBundle::equilateral_triangle(
            meshes,
            &mat_handle_white,
            Vec2::ZERO,
            SelectTriangle::SIZE,
            Some(1),
        );
        select_triangle_components.visibility.visible = false;
        let select_triangle_entity = commands
            .spawn(select_triangle_components)
            .insert(camera_manager.layer_2d)
            .insert(SelectTriangle)
            .insert(DefaultDraw)
            .id();
        input_manager.select_triangle_entity = Some(select_triangle_entity);
    }

    // select line
    {
        let mut select_line_components = create_2d_edge_line(
            meshes,
            &mat_handle_white,
            Vec2::ZERO,
            Vec2::X,
            0.0,
            SelectLine::THICKNESS,
        );
        select_line_components.visibility.visible = false;
        let select_line_entity = commands
            .spawn(select_line_components)
            .insert(camera_manager.layer_2d)
            .insert(SelectLine)
            .insert(DefaultDraw)
            .id();
        input_manager.select_line_entity = Some(select_line_entity);
    }
}

fn setup_3d_scene(
    commands: &mut Commands,
    camera_manager: &mut CameraManager,
    texture_size: &Vec2,
    canvas_texture_handle: Handle<CpuTexture2D>,
) {
    camera_manager.layer_3d = RenderLayers::layer(3);

    // Ambient Light
    commands
        .spawn(AmbientLight::new(0.01, Color::WHITE))
        .insert(camera_manager.layer_3d);
    // directional light
    commands
        .spawn(DirectionalLight {
            direction: Vec3::new(60.0, 60.0, 90.0),
            color: Color::WHITE,
            intensity: 0.2,
            ..Default::default()
        })
        .insert(camera_manager.layer_3d);

    // Camera
    let camera_entity = commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: Some(Viewport::new_at_origin(
                    texture_size.x as u32,
                    texture_size.y as u32,
                )),
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Image(canvas_texture_handle),
                ..Default::default()
            },
            transform: Transform::from_xyz(500.0, 0.0, 0.0) // from front
                .looking_at(Vec3::ZERO, Vec3::Z),
            projection: Projection::Orthographic(OrthographicProjection::new(0.0, 1000.0)),
        })
        .insert(camera_manager.layer_3d)
        .id();
    camera_manager.camera_3d = Some(camera_entity);
}

fn new_render_texture(
    texture_size: &Vec2,
    textures: &mut Storage<CpuTexture2D>,
    user_textures: &mut EguiUserTextures,
) -> Handle<CpuTexture2D> {
    // This is the texture that will be rendered to.
    let texture = CpuTexture2D::from_size(texture_size.x as u32, texture_size.y as u32);

    let texture_handle = textures.add(texture);
    user_textures.add_texture(&texture_handle);

    texture_handle
}
