use bevy_ecs::{
    change_detection::ResMut,
    entity::Entity,
    event::EventReader,
    prelude::{Commands, NextState, Query},
};

use game_app_common::AppState;

use game_engine::{
    logging::info,
    render::{
        base::{CpuMaterial, CpuMesh},
        components::{RenderLayer, RenderLayers},
    },
    storage::Storage,
    ui::UiManager,
    world::WorldConnectEvent,
};

use crate::systems::scene_setup;

pub fn connect_events(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut meshes: ResMut<Storage<CpuMesh>>,
    mut materials: ResMut<Storage<CpuMaterial>>,
    mut ui_manager: ResMut<UiManager>,
    render_layer_q: Query<(Entity, &RenderLayer)>,

    mut event_reader: EventReader<WorldConnectEvent>,
) {
    // this one loop will only run once
    for _ in event_reader.read() {
        info!("received Connect to World Server!");

        // despawning all entities with RenderLayer(0)
        let render_layer_0 = RenderLayers::layer(0);
        for (entity, layer) in render_layer_q.iter() {
            if *layer == render_layer_0 {
                commands.entity(entity).despawn();
            }
        }

        // setup walker scene
        scene_setup::scene_setup(&mut commands, &mut meshes, &mut materials);

        // disable ui
        ui_manager.disable_ui();

        // set to appropriate AppState
        next_state.set(AppState::InGame);
        return;
    }
}
