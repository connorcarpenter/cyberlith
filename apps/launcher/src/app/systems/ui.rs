use bevy_ecs::{
    event::{Event,EventWriter, EventReader},
    system::{Commands, ResMut},
};
use bevy_log::info;

use game_engine::{
    asset::{
        embedded_asset_event, AssetHandle, AssetId, AssetManager, EmbeddedAssetEvent, UiData,
    },
    render::{
        base::Color,
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation,
            OrthographicProjection, Projection,
            RenderLayers, RenderTarget,
        },
    },
};

use crate::app::{
    resources::Global,
};

#[derive(Event)]
pub struct StartButtonEvent;

#[derive(Event)]
pub struct ContinueButtonEvent;

pub fn ui_setup(
    mut commands: Commands,
    mut global: ResMut<Global>,
    mut asset_manager: ResMut<AssetManager>,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
) {
    // TODO: use some kind of catalog here?
    embedded_asset_events.send(embedded_asset_event!("../embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("../embedded/34mvvk")); // verdana icon
    embedded_asset_events.send(embedded_asset_event!("../embedded/tpp7za")); // main ui

    // render_layer
    let layer = RenderLayers::layer(1);

    // ui
    let ui_handle = AssetHandle::<UiData>::new(AssetId::from_str("tpp7za").unwrap()); // TODO: use some kind of catalog?
    let ui_entity = commands.spawn(ui_handle).insert(layer).id();

    asset_manager.register_event::<StartButtonEvent>(ui_entity, ui_handle, "start_button");
    asset_manager.register_event::<ContinueButtonEvent>(ui_entity, ui_handle, "continue_button");

    // light
    commands
        .spawn(AmbientLight::new(1.0, Color::WHITE))
        .insert(layer);

    // camera
    let camera_id = commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: None, // this will set later
                target: RenderTarget::Screen,
                clear_operation: ClearOperation::none(),
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

    global.camera_ui = camera_id;
}

pub fn handle_events(
    mut start_btn_rdr: EventReader<StartButtonEvent>,
    mut continue_btn_rdr: EventReader<ContinueButtonEvent>
) {
    for _ in start_btn_rdr.read() {
        info!("start button clicked!");
    }
    for _ in continue_btn_rdr.read() {
        info!("continue button clicked!");
    }
}