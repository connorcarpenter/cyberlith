use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::EventWriter,
    prelude::{Local, With},
    system::{Commands, Query, Res, ResMut},
};
use bevy_log::info;

use game_engine::{
    asset::{
        embedded_asset_event, AssetHandle, AssetId, AssetManager, EmbeddedAssetEvent, IconData,
    },
    math::Vec3,
    render::{
        base::{Color, CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
            OrthographicProjection, PerspectiveProjection, PointLight, Projection, RenderLayer,
            RenderLayers, RenderObjectBundle, RenderTarget, Transform, Viewport, Visibility,
        },
        resources::{RenderFrame, Time},
        shapes, Window,
    },
    storage::{Handle, Storage},
    ui::Ui,
};

use crate::app::{
    resources::Global,
    systems::ui::{init_ui, json_read_ui, json_write_ui, bits_read_ui, bits_write_ui},
};

pub fn scene_setup(
    mut commands: Commands,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
) {
    // TODO: use some kind of catalog here
    embedded_asset_events.send(embedded_asset_event!("../embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("../embedded/34mvvk")); // verdana icon

    let ui_camera = setup_ui(&mut commands);

    commands.insert_resource(Global::new(ui_camera));
}

fn setup_ui(commands: &mut Commands) -> Entity {
    // render_layer
    let layer = RenderLayers::layer(1);

    // ambient light
    commands
        .spawn(AmbientLight::new(1.0, Color::WHITE))
        .insert(layer);

    // camera
    let camera_id = commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: None,
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Screen,
                ..Default::default()
            },
            projection: Projection::Orthographic(OrthographicProjection {
                near: 0.0,
                far: 2000.0,
            }),
            ..Default::default()
        })
        .insert(layer)
        .id();

    // ui

    let text_handle = AssetHandle::<IconData>::new(AssetId::from_str("34mvvk").unwrap()); // TODO: use some kind of catalog

    let ui = init_ui(&text_handle);

    let ui_bytes = json_write_ui(ui);
    info!("json byte count: {:?}", ui_bytes.len());
    let ui = json_read_ui(ui_bytes);

    let ui_bytes = bits_write_ui(ui);
    info!("bits byte count: {:?}", ui_bytes.len());
    let ui = bits_read_ui(ui_bytes);

    let _ui_entity = commands.spawn(ui).insert(layer).id();

    camera_id
}

pub fn scene_draw(
    mut render_frame: ResMut<RenderFrame>,
    asset_manager: Res<AssetManager>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // UIs
    mut uis_q: Query<(&mut Ui, Option<&RenderLayer>)>,
    // Lights
    ambient_lights_q: Query<(&AmbientLight, Option<&RenderLayer>)>,
) {
    // Aggregate Cameras
    for (camera, transform, projection, render_layer_opt) in cameras_q.iter() {
        if !camera.is_active {
            continue;
        }
        render_frame.draw_camera(render_layer_opt, camera, transform, projection);
    }

    // Aggregate Ambient Lights
    for (ambient_light, render_layer_opt) in ambient_lights_q.iter() {
        render_frame.draw_ambient_light(render_layer_opt, ambient_light);
    }

    // Aggregate UIs
    for (mut ui, render_layer_opt) in uis_q.iter_mut() {
        ui.draw(&mut render_frame, render_layer_opt, &asset_manager);
    }
}

pub fn handle_viewport_resize(
    global: Res<Global>,
    mut window: ResMut<Window>,
    mut cameras_q: Query<(&mut Camera, &mut Transform)>,
) {
    // sync camera viewport to window
    if !window.did_change() {
        return;
    }
    window.clear_change();
    let Some(window_res) = window.get() else {
        return;
    };

    // resize ui camera
    if let Ok((mut camera, mut transform)) = cameras_q.get_mut(global.camera_ui) {
        let should_change = if let Some(viewport) = camera.viewport.as_mut() {
            *viewport != window_res.logical_size
        } else {
            true
        };
        if should_change {
            let new_viewport = Viewport::new_at_origin(
                window_res.logical_size.width,
                window_res.logical_size.height,
            );
            camera.viewport = Some(new_viewport);

            //info!("resize window detected. new viewport: (x: {:?}, y: {:?}, width: {:?}, height: {:?})", new_viewport.x, new_viewport.y, new_viewport.width, new_viewport.height);

            *transform = Transform::from_xyz(
                new_viewport.width as f32 * 0.5,
                new_viewport.height as f32 * 0.5,
                1000.0,
            )
            .looking_at(
                Vec3::new(
                    new_viewport.width as f32 * 0.5,
                    new_viewport.height as f32 * 0.5,
                    0.0,
                ),
                Vec3::NEG_Y,
            );
        }
    }
}
