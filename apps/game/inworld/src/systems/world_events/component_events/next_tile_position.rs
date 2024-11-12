
use bevy_ecs::{
    change_detection::ResMut,
    event::EventReader,
    prelude::{Commands, Query},
    system::Res,
};

use game_engine::{
    logging::{info, warn},
    math::{Quat, Vec3},
    naia::{sequence_greater_than, Tick},
    render::components::{RenderLayers, Transform, Visibility},
    time::Instant,
    world::{
        // behavior as shared_behavior,
        components::{NextTilePosition, PhysicsController},
        WorldClient, WorldInsertComponentEvent, WorldRemoveComponentEvent,
        WorldUpdateComponentEvent,
    },
};

use crate::{
    components::{AnimationState, ClientTileMovement, Confirmed, RenderPosition},
    resources::{Global, InputManager, TickTracker},
    systems::world_events::{process_tick, PredictionEvents},
};

pub fn insert_next_tile_position_events(
    client: WorldClient,
    mut commands: Commands,
    next_tile_position_q: Query<&NextTilePosition>,
    mut prediction_events: ResMut<PredictionEvents>,
    mut event_reader: EventReader<WorldInsertComponentEvent<NextTilePosition>>,
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
            .insert(ClientTileMovement::new_stopped(false, next_tile_position))
            .insert(PhysicsController::new(next_tile_position))
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
    client: WorldClient,
    global: Res<Global>,
    tick_tracker: Res<TickTracker>,
    mut input_manager: ResMut<InputManager>,
    mut event_reader: EventReader<WorldUpdateComponentEvent<NextTilePosition>>,
    next_tile_position_q: Query<&NextTilePosition>,
    mut tile_movement_q: Query<(&mut ClientTileMovement, &mut PhysicsController, &mut RenderPosition, &mut AnimationState)>,
) {
    // When we receive a new Position update for the Player's Entity,
    // we must ensure the Client-side Prediction also remains in-sync
    // So we roll the Prediction back to the authoritative Server state
    // and then execute all Player Commands since that tick, using the CommandHistory helper struct

    let mut events = Vec::new();
    for event in event_reader.read() {
        let server_tick = event.tick.wrapping_sub(1); // TODO: this shouldn't be necessary to sync!
        let updated_entity = event.entity;

        events.push((server_tick, updated_entity));
    }

    if events.is_empty() {
        return;
    }

    warn!("ROLLBACK!");

    for (update_tick, updated_entity) in &events {
        let Ok(next_tile_position) = next_tile_position_q.get(*updated_entity) else {
            panic!(
                "failed to get updated components for entity: {:?}",
                updated_entity
            );
        };
        let Ok((mut tile_movement, _, _, _)) = tile_movement_q.get_mut(*updated_entity) else {
            panic!(
                "failed to get tile movement q for entity: {:?}",
                updated_entity
            );
        };
        tile_movement.recv_updated_next_tile_position(*update_tick, &next_tile_position);
    }

    let Some(owned_entity) = &global.owned_entity else {
        warn!("---");
        return;
    };
    let mut latest_tick: Option<Tick> = None;
    let confirmed_entity = owned_entity.confirmed;
    let predicted_entity = owned_entity.predicted;

    for (server_tick, updated_entity) in events {
        // If entity is owned
        if updated_entity == confirmed_entity {
            if let Some(last_tick) = &mut latest_tick {
                if sequence_greater_than(server_tick, *last_tick) {
                    *last_tick = server_tick;
                }
            } else {
                latest_tick = Some(server_tick);
            }
        }
    }

    let Some(server_tick) = latest_tick else {
        warn!("---");
        return;
    };

    info!("Update received for Server Tick: {:?} (which is 1 less than came through in update event)", server_tick);

    let Ok(
        [(
            confirmed_tile_movement,
            confirmed_physics,
            confirmed_render_position,
            _,
        ), (
            mut predicted_tile_movement,
            mut predicted_physics,
            mut predicted_render_position,
            mut predicted_animation_state,
        )],
    ) = tile_movement_q.get_many_mut([confirmed_entity, predicted_entity])
    else {
        panic!(
            "failed to get components for entities: {:?}, {:?}",
            confirmed_entity, predicted_entity
        );
    };

    let current_tick = server_tick;

    let last_processed_server_tick = tick_tracker.last_processed_server_tick();
    if last_processed_server_tick != current_tick {
        warn!("Discrepancy! Last Processed Server Tick: {:?}. Server Update Tick: {:?}", last_processed_server_tick, current_tick);
    }

    // ROLLBACK CLIENT: Replay all stored commands

    // Set to authoritative state
    predicted_tile_movement.recv_rollback(&confirmed_tile_movement);
    predicted_physics.recv_rollback(&confirmed_physics);
    predicted_render_position.recv_rollback(&confirmed_render_position);

    // PREDICTION ROLLBACK

    let replay_commands = input_manager.pop_command_replays(current_tick);

    // process commands
    for (command_tick, outgoing_command_opt) in replay_commands {

        // info!("Replay Command. Tick: {:?}. MoveDir: {:?}. Dis: {:?}", command_tick, outgoing_command_opt.as_ref().map(|c| c.get_move()), predicted_tile_movement.get_dis());

        // process command
        input_manager.recv_incoming_command(command_tick, outgoing_command_opt);

        // process movement
        let player_command = input_manager.pop_incoming_command(command_tick);
        process_tick(
            false,
            true,
            command_tick,
            player_command,
            &mut predicted_tile_movement,
            &mut predicted_physics,
            &mut predicted_render_position,
            &mut predicted_animation_state,
        );
    }
    warn!("---");

    predicted_render_position.advance_millis(&client, 0);
    predicted_tile_movement.finish_rollback();
}

pub fn remove_next_tile_position_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NextTilePosition>>,
) {
    for _event in event_reader.read() {
        info!("removed NextTilePosition component from entity");
    }
}
