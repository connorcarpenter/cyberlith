use bevy_ecs::system::{Commands, Res, ResMut};
use bevy_log::info;

use math::{Vec2, Vec3};
use render_api::{
    Assets,
    base::{Color, CpuTexture2D},
    components::{
        AmbientLight, Camera, CameraBundle, ClearOperation, OrthographicProjection, PointLight,
        Projection, RenderLayers, RenderTarget, Transform, Viewport,
    }, Handle,
};
use render_egui::EguiUserTextures;

use crate::app::{config::AppConfig, resources::canvas_state::CanvasState};

pub fn setup(
    config: Res<AppConfig>,
    mut commands: Commands,
    mut canvas_state: ResMut<CanvasState>,
    mut textures: ResMut<Assets<CpuTexture2D>>,
    mut user_textures: ResMut<EguiUserTextures>,
) {
    info!("Environment: {}", config.general.env_name);

    // Canvas Texture
    let texture_size = Vec2::new(1130.0, 672.0);
    let canvas_texture_handle =
        new_render_texture(&texture_size, &mut textures, &mut user_textures);
    canvas_state.set_canvas_texture(canvas_texture_handle.clone());

    setup_3d_scene(
        &mut commands,
        &mut canvas_state,
        &texture_size,
        canvas_texture_handle,
    );
    setup_2d_scene(
        &mut commands,
        &mut canvas_state,
        &texture_size,
        canvas_texture_handle,
    );
}

fn setup_2d_scene(
    commands: &mut Commands,
    canvas_state: &mut CanvasState,
    texture_size: &Vec2,
    canvas_texture_handle: Handle<CpuTexture2D>,
) {
    canvas_state.layer_2d = RenderLayers::layer(2);

    // light
    commands
        .spawn(AmbientLight {
            intensity: 1.0,
            color: Color::WHITE,
            ..Default::default()
        })
        .insert(canvas_state.layer_2d);

    // camera
    let mut camera_bundle = CameraBundle::new_2d(&Viewport::new_at_origin(
        texture_size.x as u32,
        texture_size.y as u32,
    ));
    camera_bundle.camera.target = RenderTarget::Image(canvas_texture_handle);
    camera_bundle.camera.is_active = false;
    camera_bundle.camera.order = 1;
    let camera_entity = commands.spawn(camera_bundle).insert(canvas_state.layer_2d).id();

    canvas_state.camera_2d = Some(camera_entity);
}

fn setup_3d_scene(
    commands: &mut Commands,
    canvas_state: &mut CanvasState,
    texture_size: &Vec2,
    canvas_texture_handle: Handle<CpuTexture2D>,
) {
    canvas_state.layer_3d = RenderLayers::layer(3);

    // Ambient Light
    commands
        .spawn(AmbientLight::new(0.01, Color::WHITE))
        .insert(canvas_state.layer_3d);
    commands
        .spawn(PointLight {
            position: Vec3::new(60.0, 60.0, 90.0),
            color: Color::WHITE,
            intensity: 0.2,
            ..Default::default()
        })
        .insert(canvas_state.layer_3d);

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
            projection: Projection::Orthographic(OrthographicProjection::default()),
        })
        .insert(canvas_state.layer_3d)
        .id();
    canvas_state.camera_3d = Some(camera_entity);
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
