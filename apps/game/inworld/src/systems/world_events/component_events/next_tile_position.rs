use std::collections::HashMap;

use bevy_ecs::{
    change_detection::ResMut,
    event::EventReader,
    prelude::{Commands, Query},
    system::Res,
};

use game_engine::{
    logging::info,
    math::{Quat, Vec3},
    render::components::{RenderLayers, Transform, Visibility},
    time::Instant,
};

use game_app_network::{
    naia::sequence_greater_than,
    world::{
        // behavior as shared_behavior,
        components::{NetworkedTileTarget, PhysicsController},
        WorldClient,
        WorldInsertComponentEvent,
        WorldRemoveComponentEvent,
        WorldUpdateComponentEvent,
    },
};

use crate::{
    components::{AnimationState, Confirmed, ConfirmedTileMovement, RenderPosition, TickSkipper},
    resources::{RollbackManager, TickTracker},
    systems::world_events::PredictionEvents,
};

pub fn insert_next_tile_position_events(
    client: WorldClient,
    mut commands: Commands,
    next_tile_position_q: Query<&NetworkedTileTarget>,
    mut prediction_events: ResMut<PredictionEvents>,
    mut event_reader: EventReader<WorldInsertComponentEvent<NetworkedTileTarget>>,
) {
    for event in event_reader.read() {
        let now = Instant::now();
        let server_tick = client.server_tick().unwrap();
        let server_tick_instant = client
            .tick_to_instant(server_tick)
            .expect("failed to convert tick to instant");
        let entity = event.entity;

        info!(
            "received Inserted Component: `NextTilePosition` from World Server! (entity: {:?})",
            entity
        );

        let next_tile_position = next_tile_position_q.get(entity).unwrap();

        prediction_events.read_insert_position_event(&now, &entity);

        let layer = RenderLayers::layer(0);

        commands
            .entity(entity)
            // Insert Position stuff
            .insert(ConfirmedTileMovement::new_stopped(next_tile_position))
            .insert(PhysicsController::new(next_tile_position))
            .insert(TickSkipper::new())
            // Insert other Rendering Stuff
            .insert(AnimationState::new())
            .insert(RenderPosition::new(
                next_tile_position,
                server_tick,
                server_tick_instant,
            ))
            .insert(layer)
            .insert(Visibility::default())
            .insert(
                Transform::from_translation(Vec3::splat(0.0))
                    .with_rotation(Quat::from_rotation_z(f32::to_radians(90.0))),
            )
            // mark as Confirmed
            .insert(Confirmed);
    }
}

pub fn update_next_tile_position_events(
    tick_tracker: Res<TickTracker>,
    mut rollback_manager: ResMut<RollbackManager>,
    mut event_reader: EventReader<WorldUpdateComponentEvent<NetworkedTileTarget>>,
    mut updated_q: Query<(
        &NetworkedTileTarget,
        &mut ConfirmedTileMovement,
        &mut PhysicsController,
        &mut RenderPosition,
        &mut AnimationState,
    )>,
    mut tick_skipper_q: Query<&mut TickSkipper>,
) {
    let mut events = HashMap::new();
    for event in event_reader.read() {
        let server_tick = event.tick;
        let updated_entity = event.entity;

        if !events.contains_key(&updated_entity) {
            events.insert(updated_entity, server_tick);
        } else {
            let existing_tick = events.get(&updated_entity).unwrap();
            if sequence_greater_than(server_tick, *existing_tick) {
                events.insert(updated_entity, server_tick);
            }
        }
    }

    if events.is_empty() {
        return;
    }

    for (updated_entity, update_tick) in &events {
        let Ok((
            next_tile_position,
            mut tile_movement,
            mut physics,
            mut render_position,
            mut animation_state,
        )) = updated_q.get_mut(*updated_entity)
        else {
            panic!(
                "failed to get components q for updated entity: {:?}",
                updated_entity
            );
        };

        tile_movement.recv_updated_next_tile_position(
            &tick_tracker,
            *update_tick,
            &next_tile_position,
            &mut physics,
            &mut render_position,
            &mut animation_state,
        );

        let Ok(mut tick_skipper) = tick_skipper_q.get_mut(*updated_entity) else {
            panic!(
                "failed to get tick_skipper q for entity: {:?}",
                updated_entity
            );
        };
        // info!("entity: {:?}, queueing skipped tick: {:?}", updated_entity, update_tick);
        tick_skipper.queue_skipped_tick(*update_tick);
    }

    rollback_manager.add_events(events);
}

pub fn remove_next_tile_position_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NetworkedTileTarget>>,
) {
    for _event in event_reader.read() {
        info!("removed NextTilePosition component from entity");
    }
}
