use std::collections::HashMap;

use bevy_ecs::{
    change_detection::Mut,
    entity::Entity,
    prelude::Query,
    system::{Res, ResMut, SystemState, Resource},
    world::World,
};

use game_engine::{logging::warn, asset::{AssetHandle, AssetManager, UnitData}};

use game_app_network::{
    naia::{sequence_greater_than, Tick},
    world::{
        components::{PhysicsController, NetworkedLastCommand, TileMovementType},
        WorldClient,
    },
};

use crate::{
    components::{AnimationState, PredictedTileMovement, RenderPosition},
    resources::{PredictedWorld, Global, TickTracker, InputManager},
    systems::world_events::process_tick,
};

#[derive(Resource)]
pub struct RollbackManager {
    events: Option<Tick>,
}

impl Default for RollbackManager {
    fn default() -> Self {
        Self {
            events: None,
        }
    }
}

impl RollbackManager {
    pub fn add_events(&mut self, events: HashMap<Entity, Tick>) {
        for (_, tick) in events {
            self.add_event(tick);
        }
    }

    pub fn add_event(&mut self, tick: Tick) {
        if let Some(existing_tick) = self.events {
            if sequence_greater_than(tick, existing_tick) {
                self.events = Some(tick);
            }
        } else {
            self.events = Some(tick);
        }
    }

    // Used as a system
    pub(crate) fn execute_rollback(
        main_world: &mut World,
    ) {
        let (current_tick, owned_entity_opt) = {
            let mut main_system_state: SystemState<(
                Res<Global>,
                ResMut<Self>,
                Res<TickTracker>,
            )> = SystemState::new(main_world);
            let (
                global,
                mut me,
                tick_tracker,
            ) = main_system_state.get_mut(main_world);

            let Some(server_tick) = std::mem::take(&mut me.events) else {
                return;
            };

            let owned_entity_opt = global.owned_entity;

            warn!("ROLLBACK! (Tick: {:?})", server_tick);

            // info!(
            //     "Update received for Server Tick: {:?} (which is 1 less than came through in update event)",
            //     server_tick
            // );

            let mut current_tick = server_tick;
            if let Some(last_processed_server_tick) = tick_tracker.last_processed_server_tick() {
                if current_tick != last_processed_server_tick {
                    // TODO: just should be a warn, also do we need this?
                    panic!(
                        "Using last processed server tick: {:?}, instead of previous tick: {:?}",
                        last_processed_server_tick, current_tick
                    );
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

            // not necessary here yet
            main_system_state.apply(main_world);

            (current_tick, owned_entity_opt)
        };

        main_world.resource_scope(|main_world: &mut World, mut predicted_world: Mut<PredictedWorld>| {

            // ROLLBACK CLIENT: Replay all stored commands

            // Set to authoritative state
            predicted_world.extract(main_world);

            // PREDICTION ROLLBACK

            let mut main_system_state: SystemState<
                (
                    WorldClient,
                    Res<AssetManager>,
                    ResMut<InputManager>,
                    Query<(Entity, &AssetHandle<UnitData>, &NetworkedLastCommand)>,
                )> = SystemState::new(main_world);
            let (
                client,
                asset_manager,
                mut input_manager,
                unit_q,
            ) = main_system_state.get_mut(main_world);

            let mut predicted_system_state: SystemState<
                Query<(
                    &mut PredictedTileMovement,
                    &mut PhysicsController,
                    &mut RenderPosition,
                    &mut AnimationState,
                )>
            > = SystemState::new(predicted_world.world_mut());
            let mut predicted_unit_q = predicted_system_state.get_mut(predicted_world.world_mut());

            let replay_commands = input_manager.pop_command_replays(current_tick);

            // process commands
            for (command_tick, outgoing_command_opt) in replay_commands {
                // info!("Replay Command (Tick: {:?})", command_tick);

                // process command
                input_manager.recv_incoming_command(command_tick, outgoing_command_opt);

                // process movement
                let player_command = input_manager.pop_incoming_command(command_tick);

                //
                for (entity, unit_handle, last_command) in unit_q.iter() {
                    let animated_model_handle = asset_manager.get_unit_animated_model_handle(unit_handle).unwrap();

                    let (
                        mut predicted_tile_movement,
                        mut predicted_physics,
                        mut predicted_render_position,
                        mut predicted_animation_state,
                    ) = predicted_unit_q.get_mut(entity).unwrap();

                    let next_command = {
                        if let Some(owned_entity) = owned_entity_opt {
                            if entity == owned_entity {
                                player_command.clone()
                            } else {
                                last_command.to_player_commands()
                            }
                        } else {
                            last_command.to_player_commands()
                        }
                    };
                    let predicted_tile_movement_2: &mut PredictedTileMovement =
                        &mut predicted_tile_movement;
                    process_tick(
                        &asset_manager,
                        TileMovementType::ClientPredicted,
                        command_tick,
                        next_command,
                        predicted_tile_movement_2,
                        &mut predicted_physics,
                        &mut predicted_render_position,
                        &mut predicted_animation_state,
                        &animated_model_handle,
                    );
                }
            }
            warn!("---");

            // TODO: why is this necessary again? should refactor into a separate method
            for (entity, _, _) in unit_q.iter() {
                let (_, _, mut predicted_render_position, _) = predicted_unit_q.get_mut(entity).unwrap();
                predicted_render_position.advance_millis(&client, 0);
            }
        });
    }
}
