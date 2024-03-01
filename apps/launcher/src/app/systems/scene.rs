use bevy_ecs::{
    component::Component,
    system::{Commands, Query, Res, ResMut},
    event::EventWriter
};

use game_engine::{
    asset::{AssetId, embedded_asset_event, EmbeddedAssetEvent, IconData, TextStyle, AssetManager, AssetHandle},
    math::Vec2,
    render::{
        base::{Color, CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, CameraBundle, DirectionalLight,
            PointLight, Projection, RenderLayer, RenderLayers,
            RenderTarget, Transform, Viewport, Visibility,
        },
        resources::RenderFrame,
    },
    storage::Handle,
};

#[derive(Component)]
pub struct TextMarker;

pub fn scene_setup(
    mut commands: Commands,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>
) {
    // TODO: use some kind of catalog here
    embedded_asset_events.send(embedded_asset_event!("../embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("../embedded/34mvvk")); // verdana icon

    let layer = RenderLayers::layer(0);

    // ambient light
    commands
        .spawn(AmbientLight::new(1.0, Color::WHITE))
        .insert(layer);

    // camera
    let viewport_width = 1280.0;
    let viewport_height = 720.0;
    let mut camera_bundle = CameraBundle::new_2d(&Viewport::new_at_origin(
        viewport_width as u32,
        viewport_height as u32,
    ));
    camera_bundle.camera.target = RenderTarget::Screen;

    commands
        .spawn_empty()
        .insert(
            Transform::from_translation_2d(Vec2::splat(64.0)),
        )
        .insert(Visibility::default())
        .insert(TextMarker)
        .insert(TextStyle::new(32.0, 4.0))
        .insert(layer)
        .insert(AssetHandle::<IconData>::new(AssetId::from_str("34mvvk").unwrap())); // TODO: use some kind of catalog
}

pub fn scene_draw(
    asset_manager: Res<AssetManager>,
    mut render_frame: ResMut<RenderFrame>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // Meshes
    cpu_meshes_q: Query<(
        &Handle<CpuMesh>,
        &Handle<CpuMaterial>,
        &Transform,
        &Visibility,
        Option<&RenderLayer>,
    )>,
    icons_q: Query<(
        &AssetHandle<IconData>,
        &TextStyle,
        &Transform,
        &Visibility,
        Option<&RenderLayer>,
    )>,
    // Lights
    ambient_lights_q: Query<(&AmbientLight, Option<&RenderLayer>)>,
    point_lights_q: Query<(&PointLight, Option<&RenderLayer>)>,
    directional_lights_q: Query<(&DirectionalLight, Option<&RenderLayer>)>,
) {
    // Aggregate Cameras
    for (camera, transform, projection, render_layer_opt) in cameras_q.iter() {
        if !camera.is_active {
            continue;
        }
        render_frame.draw_camera(render_layer_opt, camera, transform, projection);
    }

    // Aggregate Point Lights
    for (point_light, render_layer_opt) in point_lights_q.iter() {
        render_frame.draw_point_light(render_layer_opt, point_light);
    }

    // Aggregate Directional Lights
    for (dir_light, render_layer_opt) in directional_lights_q.iter() {
        render_frame.draw_directional_light(render_layer_opt, dir_light);
    }

    // Aggregate Ambient Lights
    for (ambient_light, render_layer_opt) in ambient_lights_q.iter() {
        render_frame.draw_ambient_light(render_layer_opt, ambient_light);
    }

    // Aggregate Cpu Meshes
    for (mesh_handle, mat_handle, transform, visibility, render_layer_opt) in cpu_meshes_q.iter() {
        if !visibility.visible {
            continue;
        }
        render_frame.draw_mesh(render_layer_opt, mesh_handle, mat_handle, transform);
    }

    //let mut mouse_pos = input.mouse_position();

    // Aggregate Icons
    for (icon_handle, style, transform, visibility, render_layer_opt) in icons_q.iter() {
        if !visibility.visible {
            continue;
        }
        asset_manager.draw_text(&mut render_frame, icon_handle, &style, &transform.translation, render_layer_opt, "Hello, my Nina! <3");
    }
}
