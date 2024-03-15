use bevy_ecs::{
    entity::Entity,
    event::EventWriter,
    system::{Commands, Query, Res, ResMut},
};
use bevy_log::info;

use naia_serde::{BitWriter, Serde};

use asset_io::json::{Asset, AssetData, AssetMeta, UiJson};
use asset_render::{AssetMetadataSerde, UiData};
use asset_id::ETag;

// TODO: don't depend on game engine?
use game_engine::{
    asset::{
        embedded_asset_event, AssetHandle, AssetId, AssetManager, EmbeddedAssetEvent, IconData, AssetType,
    },
    math::Vec3,
    render::{
        base::Color,
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation,
            OrthographicProjection, Projection, RenderLayer,
            RenderLayers, RenderTarget, Transform, Viewport,
        },
        resources::RenderFrame,
        Window,
    },
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

    let camera = setup_scene(&mut commands);
    setup_ui(&mut commands);

    commands.insert_resource(Global::new(camera));
}

fn setup_ui(commands: &mut Commands) {

    let name = "main"; // TODO: clean this up?
    let asset_id = AssetId::get_random();

    let text_handle = AssetHandle::<IconData>::new(AssetId::from_str("34mvvk").unwrap()); // TODO: use some kind of catalog

    let ui = init_ui(&text_handle);

    // ui -> JSON bytes
    let ui_bytes = {
        let ui_json = json_write_ui(ui);
        let new_meta = AssetMeta::new(&asset_id, UiJson::CURRENT_SCHEMA_VERSION);
        let asset = Asset::new(new_meta, AssetData::Ui(ui_json));
        let ui_bytes = serde_json::to_vec_pretty(&asset)
            .unwrap();
        info!("json byte count: {:?}", ui_bytes.len());
        ui_bytes
    };

    // write JSON bytes to file
    std::fs::write(format!("output/{}.ui.json", name), &ui_bytes).unwrap();

    // JSON bytes -> ui
    let ui = {
        let asset: Asset = serde_json::from_slice(&ui_bytes).unwrap();
        let (_, data) = asset.deconstruct();
        let AssetData::Ui(ui_json) = data else {
            panic!("expected UiData");
        };
        ui_json.to_ui()
    };

    // ui -> bit-packed bytes
    let ui_bytes = bits_write_ui(ui);
    info!("bits byte count: {:?}", ui_bytes.len());

    // write bit-packed data to file
    std::fs::write(format!("output/{}", name), &ui_bytes).unwrap();

    // write metadata to file
    {
        let ui_metadata = AssetMetadataSerde::new(ETag::new_random(), AssetType::Ui);
        let mut bit_writer = BitWriter::new();
        ui_metadata.ser(&mut bit_writer);
        let metadata_bytes = bit_writer.to_bytes();
        std::fs::write(format!("output/{}.meta", name), &metadata_bytes).unwrap();
    }

    // bit-packed bytes -> ui
    let ui = bits_read_ui(ui_bytes);

    let _ui_entity = commands.spawn(ui).id();
}

fn setup_scene(commands: &mut Commands) -> Entity {
    // ambient light
    commands
        .spawn(AmbientLight::new(1.0, Color::WHITE));

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
        .id();

    camera_id
}

pub fn scene_draw(
    mut render_frame: ResMut<RenderFrame>,
    mut asset_manager: ResMut<AssetManager>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // UIs
    mut uis_q: Query<(&AssetHandle<UiData>, Option<&RenderLayer>)>,
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
    for (ui_handle, render_layer_opt) in uis_q.iter_mut() {
        asset_manager.draw_ui(&mut render_frame, render_layer_opt, ui_handle);
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
