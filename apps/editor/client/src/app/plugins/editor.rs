use bevy_app::{App, Plugin};
use bevy_ecs::query::Or;
use bevy_ecs::{
    component::Component,
    query::With,
    schedule::IntoSystemConfigs,
    system::{Commands, Local, Query, Res, ResMut, Resource},
};

use naia_bevy_client::{
    ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin, ReceiveEvents,
};

use math::{degrees, vec3, Quat, Vec3};
use render_api::{
    base::{Camera, Color, PbrMaterial, Texture2D, TriMesh, Viewport},
    shapes, AmbientLight, Assets, CameraComponent, ClearOperation, DirectionalLight, Handle,
    PointLight, RenderLayers, RenderObjectBundle, RenderTarget, Transform, Window,
};
use render_egui::{egui, EguiContext, EguiUserTextures, GUI};

use editor_proto::protocol;
use render_egui::egui::Widget;

use crate::app::network;

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
            .add_startup_system(setup)
            .add_system(rotator_system)
            .add_system(render_to_image_example_system)
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

// Marks the main pass cube, to which the material is applied.
#[derive(Component)]
struct MainPassCube;

#[derive(Resource)]
struct CubePreviewImage(Handle<Texture2D>);

fn setup(
    window: Res<Window>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<TriMesh>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
    mut images: ResMut<Assets<Texture2D>>,
) {
    // This is the texture that will be rendered to.
    let texture_width = 512;
    let texture_height = 512;
    let mut texture = Texture2D::from_size(texture_width, texture_height);

    let texture_handle = images.add(texture);
    egui_user_textures.add_image(texture_handle.clone());
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

    // Light
    commands.spawn(PointLight {
        position: Vec3::new(0.0, 0.0, 10.0),
        ..Default::default()
    });

    // Camera to Render to Image
    commands
        .spawn(CameraComponent::new(
            Camera::new_perspective(
                Viewport::new_at_origin(texture_width, texture_height),
                vec3(0.0, 0.0, 15.0),
                vec3(0.0, 0.0, 0.0),
                vec3(0.0, 1.0, 0.0),
                degrees(45.0),
                0.1,
                5000.0,
            ),
            0,
            ClearOperation::from_rgba(1.0, 1.0, 1.0, 0.0),
            RenderTarget::Image(texture_handle),
        ))
        .insert(preview_pass_layer);

    // Main pass cube.
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(TriMesh::from(shapes::Cube { size: 3.0 })),
            material: materials.add(Color::from_rgb_f32(0.5, 0.7, 0.9).into()),
            transform: Transform {
                position: Vec3::new(0.0, 0.0, 1.5),
                rotation: Quat::from_sv(-std::f32::consts::PI / 5.0, Vec3::new(1.0, 0.0, 0.0)),
            },
        })
        .insert(MainPassCube);

    // The main pass camera.
    commands.spawn(CameraComponent::new(
        Camera::new_perspective(
            window.viewport(),
            vec3(0.0, 0.0, 15.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            0.1,
            5000.0,
        ),
        1,
        ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
        RenderTarget::Screen,
    ));
}

fn render_to_image_example_system(
    cube_preview_image: Res<CubePreviewImage>,
    preview_cube_query: Query<&Handle<PbrMaterial>, With<PreviewPassCube>>,
    main_cube_query: Query<&Handle<PbrMaterial>, With<MainPassCube>>,
    mut materials: ResMut<Assets<PbrMaterial>>,
    context: Res<EguiContext>,
    mut user_textures: ResMut<EguiUserTextures>,
) {
    let cube_preview_texture_id = user_textures.image_id(&cube_preview_image.0).unwrap();
    let preview_material_handle = preview_cube_query.single();
    let preview_material = materials.get_mut(preview_material_handle).unwrap();

    let mut apply = false;
    egui::Window::new("Cube material preview").show(context.inner(), |ui| {
        ui.image(cube_preview_texture_id, [300.0, 300.0]);
        egui::Grid::new("preview").show(ui, |ui| {
            ui.label("Base color:");
            color_picker_widget(ui, &mut preview_material.albedo);
            ui.end_row();

            ui.label("Emissive:");
            color_picker_widget(ui, &mut preview_material.emissive);
            ui.end_row();

            ui.label("Perceptual roughness:");
            egui::Slider::new(&mut preview_material.roughness, 0.089..=1.0).ui(ui);
            ui.end_row();

            ui.label("Reflectance:");
            egui::Slider::new(&mut preview_material.metallic, 0.0..=1.0).ui(ui);
            ui.end_row();
        });

        apply = ui.button("Apply").clicked();
    });

    if apply {
        let material_clone = preview_material.clone();

        let main_material_handle = main_cube_query.single();
        let _ = materials.set(main_material_handle, material_clone);
    }
}

fn color_picker_widget(ui: &mut egui::Ui, color: &mut Color) -> egui::Response {
    let [r, g, b, a] = color.to_rgba_slice();
    let mut egui_color: egui::Rgba = egui::Rgba::from_srgba_unmultiplied(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    );
    let res = egui::widgets::color_picker::color_edit_button_rgba(
        ui,
        &mut egui_color,
        egui::color_picker::Alpha::Opaque,
    );
    let [r, g, b, a] = egui_color.to_srgba_unmultiplied();
    *color = [
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ]
    .into();
    res
}

// Rotates the cubes.
#[allow(clippy::type_complexity)]
fn rotator_system(
    mut query: Query<&mut Transform, Or<(With<PreviewPassCube>, With<MainPassCube>)>>,
) {
    for mut transform in &mut query {
        transform.rotate_x(1.5);
        transform.rotate_z(1.3);
    }
}
