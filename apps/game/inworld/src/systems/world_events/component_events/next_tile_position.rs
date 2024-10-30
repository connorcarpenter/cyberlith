
use bevy_ecs::{
    change_detection::ResMut,
    event::EventReader,
    prelude::{Commands, Query},
    system::Res,
};

use game_engine::{
    logging::info,
    math::{Quat, Vec3},
    naia::{sequence_greater_than, Tick},
    render::components::{RenderLayers, Transform, Visibility},
    time::Instant,
    world::{
        // behavior as shared_behavior,
        components::{NextTilePosition},
        WorldClient, WorldInsertComponentEvent, WorldRemoveComponentEvent,
        WorldUpdateComponentEvent,
    },
};

use crate::{
    components::{AnimationState, Confirmed, RenderPosition},
    resources::{Global, InputManager},
    systems::world_events::{process_tick, PredictionEvents},
};
use crate::components::ClientTileMovement;

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
    mut input_manager: ResMut<InputManager>,
    mut event_reader: EventReader<WorldUpdateComponentEvent<NextTilePosition>>,
    next_tile_position_q: Query<&NextTilePosition>,
    mut tile_movement_q: Query<(&mut ClientTileMovement, &mut RenderPosition, &mut AnimationState)>,
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

    for (update_tick, updated_entity) in &events {
        let Ok(next_tile_position) = next_tile_position_q.get(*updated_entity) else {
            panic!(
                "failed to get updated components for entity: {:?}",
                updated_entity
            );
        };
        let Ok((mut tile_movement, _, _)) = tile_movement_q.get_mut(*updated_entity) else {
            panic!(
                "failed to get tile movement q for entity: {:?}",
                updated_entity
            );
        };
        tile_movement.recv_updated_next_tile_position(*update_tick, &next_tile_position, false);
    }

    let Some(owned_entity) = &global.owned_entity else {
        return;
    };
    let mut latest_tick: Option<Tick> = None;
    let server_entity = owned_entity.confirmed;
    let client_entity = owned_entity.predicted;

    for (server_tick, updated_entity) in events {
        // If entity is owned
        if updated_entity == server_entity {
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
        return;
    };

    //info!("Update received for Server Tick: {:?} (which is 1 less than came through in update event)", server_tick);

    let Ok(
        [(
            mut server_tile_movement,
            mut server_render_position,
            mut server_animation_state,
        ), (
            mut client_tile_movement,
            mut client_render_position,
            mut client_animation_state,
        )],
    ) = tile_movement_q.get_many_mut([server_entity, client_entity])
    else {
        panic!(
            "failed to get components for entities: {:?}, {:?}",
            server_entity, client_entity
        );
    };

    let old_server_tick = client.server_tick().unwrap();
    let mut current_tick = server_tick;

    // ROLL FORWARD SERVER

    {
        while sequence_greater_than(old_server_tick, current_tick)
            || old_server_tick == current_tick
        {
            process_tick(
                true,
                true,
                current_tick,
                None,
                &mut server_tile_movement,
                &mut server_render_position,
                &mut server_animation_state,
            );
            current_tick = current_tick.wrapping_add(1);
        }
    }

    //info!("Updated Server Tick to {:?}", current_tick);

    // ROLLBACK CLIENT: Replay all stored commands

    // Set to authoritative state
    client_tile_movement.recv_rollback(server_tile_movement.inner());
    client_render_position.recv_rollback(&server_render_position);

    // PREDICTION ROLLBACK

    let replay_commands = input_manager.pop_command_replays(current_tick);

    // process commands
    //warn!("ROLLBACK!");
    for (command_tick, outgoing_command_opt) in replay_commands {

        // process command
        input_manager.recv_incoming_command(command_tick, outgoing_command_opt);

        // process movement
        let player_command = input_manager.pop_incoming_command(command_tick);
        process_tick(
            false,
            true,
            command_tick,
            player_command,
            &mut client_tile_movement,
            &mut client_render_position,
            &mut client_animation_state,
        );
    }
    //warn!("---");

    client_render_position.advance_millis(&client, 0);
}

pub fn remove_next_tile_position_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NextTilePosition>>,
) {
    for _event in event_reader.read() {
        info!("removed NextTilePosition component from entity");
    }
}
