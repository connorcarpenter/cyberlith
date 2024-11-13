use bevy_ecs::{change_detection::{ResMut, Res}, event::EventReader, prelude::Query};

use game_engine::{
    logging::info,
    time::Instant,
    world::{WorldClient,
        components::{HasMoveBuffered, PhysicsController}, WorldInsertComponentEvent, WorldRemoveComponentEvent,
        WorldUpdateComponentEvent,
    },
};

use crate::{systems::world_events::{component_events::rollback::execute_rollback, PredictionEvents}, resources::{Global, InputManager, TickTracker}, components::{AnimationState, ConfirmedTileMovement, PredictedTileMovement, RenderPosition}};

pub fn insert_has_move_buffered_events(
    mut prediction_events: ResMut<PredictionEvents>,
    mut event_reader: EventReader<WorldInsertComponentEvent<HasMoveBuffered>>,
) {
    for event in event_reader.read() {
        let now = Instant::now();
        let entity = event.entity;

        info!(
            "received Inserted Component: `HasMoveBuffered` from World Server! (entity: {:?})",
            entity
        );

        prediction_events.read_insert_hasmovebuffered_event(&now, &entity);
    }
}

pub fn update_has_move_buffered_events(
    client: WorldClient,
    global: Res<Global>,
    // tick_tracker: Res<TickTracker>,
    mut input_manager: ResMut<InputManager>,
    mut event_reader: EventReader<WorldUpdateComponentEvent<HasMoveBuffered>>,
    has_move_buffered_q: Query<&HasMoveBuffered>,
    mut predicted_tile_movement_q: Query<&mut PredictedTileMovement>,
    mut confirmed_tile_movement_q: Query<&mut ConfirmedTileMovement>,
    mut physics_q: Query<&mut PhysicsController>,
    mut render_q: Query<(
        &mut RenderPosition,
        &mut AnimationState,
    )>,
) {
    let mut events = Vec::new();
    for event in event_reader.read() {
        let server_tick = event.tick.wrapping_sub(1); // TODO: this shouldn't be necessary to sync!
        let updated_entity = event.entity;

        events.push((server_tick, updated_entity));
    }

    if events.is_empty() {
        return;
    }

    for (update_tick, updated_entity) in &events {
        let Ok(has_move_buffered) = has_move_buffered_q.get(*updated_entity) else {
            panic!(
                "failed to get updated components for entity: {:?}",
                updated_entity
            );
        };
        let Ok(mut tile_movement) = confirmed_tile_movement_q.get_mut(*updated_entity) else {
            panic!(
                "failed to get tile movement q for entity: {:?}",
                updated_entity
            );
        };
        tile_movement.recv_updated_has_move_buffered(*update_tick, &has_move_buffered);
    }

    execute_rollback(
        &client,
        &global,
        // &tick_tracker,
        &mut input_manager,
        &mut predicted_tile_movement_q,
        &mut confirmed_tile_movement_q,
        &mut physics_q,
        &mut render_q,
        events
    );
}

pub fn remove_has_move_buffered_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<HasMoveBuffered>>,
) {
    for _event in event_reader.read() {
        info!("removed HasMoveBuffered component from entity");
    }
}
