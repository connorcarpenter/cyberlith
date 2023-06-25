use bevy_ecs::system::{Commands, Res, ResMut, Resource};
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

use crate::app::{config::AppConfig, resources::global::Global};

#[derive(Resource)]
pub struct CanvasTexture(pub Handle<CpuTexture2D>);

pub fn setup(
    config: Res<AppConfig>,
    mut commands: Commands,
    mut global: ResMut<Global>,
    mut textures: ResMut<Assets<CpuTexture2D>>,
    mut user_textures: ResMut<EguiUserTextures>,
) {
    info!("Environment: {}", config.general.env_name);

    // Canvas Texture
    let texture_size = Vec2::new(1130.0, 672.0);
    let canvas_texture_handle =
        new_render_texture(&texture_size, &mut textures, &mut user_textures);
    commands.insert_resource(CanvasTexture(canvas_texture_handle.clone()));

    setup_3d_scene(
        &mut commands,
        &mut global,
        &texture_size,
        canvas_texture_handle,
    );
    setup_2d_scene(
        &mut commands,
        &mut global,
        &texture_size,
        canvas_texture_handle,
    );
}

fn setup_2d_scene(
    commands: &mut Commands,
    global: &mut Global,
    texture_size: &Vec2,
    canvas_texture_handle: Handle<CpuTexture2D>,
) {
    global.layer_2d = RenderLayers::layer(2);

    // light
    commands
        .spawn(AmbientLight {
            intensity: 1.0,
            color: Color::WHITE,
            ..Default::default()
        })
        .insert(global.layer_2d);

    // camera
    let mut camera_bundle = CameraBundle::new_2d(&Viewport::new_at_origin(
        texture_size.x as u32,
        texture_size.y as u32,
    ));
    camera_bundle.camera.target = RenderTarget::Image(canvas_texture_handle);
    camera_bundle.camera.is_active = false;
    camera_bundle.camera.order = 1;
    let camera_entity = commands.spawn(camera_bundle).insert(global.layer_2d).id();

    global.camera_2d = Some(camera_entity);
}

fn setup_3d_scene(
    commands: &mut Commands,
    global: &mut Global,
    texture_size: &Vec2,
    canvas_texture_handle: Handle<CpuTexture2D>,
) {
    global.layer_3d = RenderLayers::layer(3);

    // Ambient Light
    commands
        .spawn(AmbientLight::new(0.01, Color::WHITE))
        .insert(global.layer_3d);
    commands
        .spawn(PointLight {
            position: Vec3::new(60.0, 60.0, 90.0),
            color: Color::WHITE,
            intensity: 0.2,
            ..Default::default()
        })
        .insert(global.layer_3d);

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
            transform: Transform::from_xyz(60.0, 30.0, 60.0) // isometric-ish
                .looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Orthographic(OrthographicProjection::default()),
        })
        .insert(global.layer_3d)
        .id();
    global.camera_3d = Some(camera_entity);
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
