use std::collections::HashMap;

use bevy_ecs::{system::{Res, ResMut, Resource}, prelude::Query, entity::Entity};

use game_engine::logging::{warn};
use game_app_network::{world::{components::{PhysicsController, TileMovementType}, WorldClient}, naia::{sequence_greater_than, Tick}};

use crate::{resources::TickTracker, systems::world_events::process_tick, resources::{Global, InputManager}, components::{AnimationState, ConfirmedTileMovement, PredictedTileMovement, RenderPosition}};

#[derive(Resource)]
pub struct RollbackManager {
    events: HashMap<Entity, Tick>,
}

impl Default for RollbackManager {
    fn default() -> Self {
        Self {
            events: HashMap::new(),
        }
    }
}

impl RollbackManager {

    pub fn add_events(&mut self, events: HashMap<Entity, Tick>) {
        for (entity, tick) in events {
            if self.events.contains_key(&entity) {
                let existing_tick = self.events.get(&entity).unwrap();
                if sequence_greater_than(tick, *existing_tick) {
                    self.events.insert(entity, tick);
                }
            } else {
                self.events.insert(entity, tick);
            }
        }
    }

    // Used as a system
    pub(crate) fn execute_rollback(
        client: WorldClient,
        global: Res<Global>,
        mut me: ResMut<Self>,
        tick_tracker: Res<TickTracker>,
        mut input_manager: ResMut<InputManager>,
        mut predicted_tile_movement_q: Query<&mut PredictedTileMovement>,
        mut confirmed_tile_movement_q: Query<&mut ConfirmedTileMovement>,
        mut components_q: Query<(&mut PhysicsController, &mut RenderPosition, &mut AnimationState)>,
    ) {
        let events = std::mem::take(&mut me.events);

        let Some(owned_entity) = &global.owned_entity else {
            // warn!("---");
            return;
        };

        let confirmed_entity = owned_entity.confirmed;

        let Some(server_tick) = events.get(&confirmed_entity) else {
            return;
        };
        let server_tick = *server_tick;

        let predicted_entity = owned_entity.predicted;

        warn!("ROLLBACK! (Tick: {:?})", server_tick);

        // info!(
        //     "Update received for Server Tick: {:?} (which is 1 less than came through in update event)",
        //     server_tick
        // );

        let Ok(confirmed_tile_movement) = confirmed_tile_movement_q.get_mut(confirmed_entity) else {
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
        let Ok(
            [(confirmed_physics, confirmed_render_position, confirmed_animation_state), (mut predicted_physics, mut predicted_render_position, mut predicted_animation_state)],
        ) = components_q.get_many_mut([confirmed_entity, predicted_entity])
        else {
            panic!(
                "failed to get components for entities: {:?}, {:?}",
                confirmed_entity, predicted_entity
            );
        };

        let mut current_tick = server_tick;
        if let Some(last_processed_server_tick) = tick_tracker.last_processed_server_tick() {
            if current_tick != last_processed_server_tick {
                // TODO: just should be a warn, also do we need this?
                panic!("Using last processed server tick: {:?}, instead of previous tick: {:?}", last_processed_server_tick, current_tick);
            }
            current_tick = last_processed_server_tick;
        }

        // // roll server state forward possibly
        // if let Some(last_processed_server_tick) = tick_tracker.last_processed_server_tick() {
        //     if last_processed_server_tick != current_tick {
        //         if sequence_greater_than(last_processed_server_tick, current_tick) {
        //             info!(
        //                 "Rolling forward server state from {:?} to {:?}",
        //                 current_tick, last_processed_server_tick,
        //             );
        //
        //             while current_tick != last_processed_server_tick {
        //
        //                 current_tick = current_tick.wrapping_add(1);
        //
        //                 // process movement
        //                 let confirmed_tile_movement_2: &mut ConfirmedTileMovement = &mut confirmed_tile_movement;
        //                 process_tick(
        //                     TileMovementType::ClientConfirmed,
        //                     current_tick,
        //                     None,
        //                     confirmed_tile_movement_2,
        //                     &mut confirmed_physics,
        //                     &mut confirmed_render_position,
        //                     &mut confirmed_animation_state,
        //                 );
        //             }
        //
        //         } else {
        //             warn!(
        //                 "Discrepancy! Last Processed Server Tick: {:?}. Server Update Tick: {:?}",
        //                 last_processed_server_tick, current_tick
        //             );
        //         }
        //     }
        // }

        // ROLLBACK CLIENT: Replay all stored commands

        // Set to authoritative state
        let confirmed_tile_movement: &ConfirmedTileMovement = &confirmed_tile_movement;
        *predicted_tile_movement = PredictedTileMovement::from(confirmed_tile_movement);
        predicted_physics.recv_rollback(current_tick, &confirmed_physics);
        predicted_render_position.recv_rollback(&confirmed_render_position);
        predicted_animation_state.recv_rollback(&confirmed_animation_state);

        // PREDICTION ROLLBACK

        let replay_commands = input_manager.pop_command_replays(current_tick);

        // process commands
        for (command_tick, outgoing_command_opt) in replay_commands {
            // info!("Replay Command (Tick: {:?})", command_tick);

            // process command
            input_manager.recv_incoming_command(command_tick, outgoing_command_opt);

            // process movement
            let player_command = input_manager.pop_incoming_command(command_tick);

            let predicted_tile_movement_2: &mut PredictedTileMovement = &mut predicted_tile_movement;
            process_tick(
                TileMovementType::ClientPredicted,
                command_tick,
                player_command,
                predicted_tile_movement_2,
                &mut predicted_physics,
                &mut predicted_render_position,
                &mut predicted_animation_state,
            );
        }
        warn!("---");

        // TODO: why is this necessary again? should refactor into a separate method
        predicted_render_position.advance_millis(&client, 0);
    }
}
