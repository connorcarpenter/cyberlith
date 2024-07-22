use bevy_ecs::{entity::Entity, event::EventReader, prelude::NextState, system::{Commands, Query, ResMut}};

use game_engine::{
    logging::info,
    render::{base::{CpuMaterial, CpuMesh}, components::{RenderLayer, RenderLayers}},
    storage::Storage,
    ui::UiManager,
    world::{
        components::Position, WorldConnectEvent,
        WorldInsertComponentEvent, WorldSpawnEntityEvent,
    },
};

use game_app_common::AppState;

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
        scene_setup::scene_setup(
            &mut commands,
            &mut meshes,
            &mut materials,
        );

        // disable ui
        ui_manager.disable_ui();

        // set to appropriate AppState
        next_state.set(AppState::InGame);
        return;
    }
}

pub fn spawn_entity_events(mut event_reader: EventReader<WorldSpawnEntityEvent>) {
    for event in event_reader.read() {
        info!(
            "received Spawn Entity from World Server! (entity: {:?})",
            event.entity
        );
    }
}

pub fn insert_position_events(
    mut event_reader: EventReader<WorldInsertComponentEvent<Position>>,
) {
    for event in event_reader.read() {
        info!(
            "received Inserted Component: `Position` from World Server! (entity: {:?})",
            event.entity
        );
    }
}