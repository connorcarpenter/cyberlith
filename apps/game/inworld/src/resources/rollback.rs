
use bevy_ecs::{system::{Res, ResMut, Resource}, prelude::Query, entity::Entity};

use game_engine::{world::{components::{PhysicsController, TileMovementType}, WorldClient}, naia::{sequence_greater_than, Tick}, logging::warn};

use crate::{systems::world_events::process_tick, resources::{Global, InputManager}, components::{AnimationState, ConfirmedTileMovement, PredictedTileMovement, RenderPosition}};

#[derive(Resource)]
pub struct RollbackManager {
    events: Vec<(Tick, Entity)>,
}

impl Default for RollbackManager {
    fn default() -> Self {
        Self {
            events: Vec::new(),
        }
    }
}

impl RollbackManager {

    pub fn add_events(&mut self, mut events: Vec<(Tick, Entity)>) {
        self.events.append(&mut events);
    }

    // Used as a system
    pub(crate) fn execute_rollback(
        client: WorldClient,
        global: Res<Global>,
        mut me: ResMut<Self>,
        // tick_tracker: &TickTracker,
        mut input_manager: ResMut<InputManager>,
        mut predicted_tile_movement_q: Query<&mut PredictedTileMovement>,
        confirmed_tile_movement_q: Query<&ConfirmedTileMovement>,
        mut physics_q: Query<&mut PhysicsController>,
        mut render_q: Query<(&mut RenderPosition, &mut AnimationState)>,
    ) {
        let events = std::mem::take(&mut me.events);

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
            return;
        };

        warn!("ROLLBACK!");

        // info!(
        //     "Update received for Server Tick: {:?} (which is 1 less than came through in update event)",
        //     server_tick
        // );

        let Ok(confirmed_tile_movement) = confirmed_tile_movement_q.get(confirmed_entity) else {
            panic!(
                "failed to get confirmed tile movement for entity: {:?}",
                confirmed_entity
            );
        };
        let Ok(mut predicted_tile_movement) = predicted_tile_movement_q.get_mut(predicted_entity)
        else {
            panic!(
                "failed to get predicted tile movement for entity: {:?}",
                predicted_entity
            );
        };
        let Ok([confirmed_physics, mut predicted_physics]) = physics_q.get_many_mut([confirmed_entity, predicted_entity])
        else {
            panic!(
                "failed to get physics for entities: {:?}, {:?}",
                confirmed_entity, predicted_entity
            );
        };
        let Ok(
            [(confirmed_render_position, confirmed_animation_state), (mut predicted_render_position, mut predicted_animation_state)],
        ) = render_q.get_many_mut([confirmed_entity, predicted_entity])
        else {
            panic!(
                "failed to get components for entities: {:?}, {:?}",
                confirmed_entity, predicted_entity
            );
        };

        let current_tick = server_tick;

        // let last_processed_server_tick = tick_tracker.last_processed_server_tick();
        // if last_processed_server_tick != current_tick {
        //     warn!(
        //         "Discrepancy! Last Processed Server Tick: {:?}. Server Update Tick: {:?}",
        //         last_processed_server_tick, current_tick
        //     );
        // }

        // ROLLBACK CLIENT: Replay all stored commands

        // Set to authoritative state
        let mut rollback_tile_movement = PredictedTileMovement::from(confirmed_tile_movement);
        predicted_physics.recv_rollback(&confirmed_physics);
        predicted_render_position.recv_rollback(&confirmed_render_position);
        predicted_animation_state.recv_rollback(&confirmed_animation_state);

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
                TileMovementType::ClientPredicted,
                command_tick,
                player_command,
                &mut rollback_tile_movement,
                &mut predicted_physics,
                &mut predicted_render_position,
                &mut predicted_animation_state,
            );
        }
        warn!("---");

        predicted_tile_movement.recv_rollback(rollback_tile_movement);

        predicted_render_position.advance_millis(&client, 0);
    }
}
