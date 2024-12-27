use std::collections::HashMap;

use bevy_ecs::{
    change_detection::Mut,
    entity::Entity,
    event::EventReader,
    prelude::{Query, Resource, World},
    system::{Res, SystemState},
};

use naia_bevy_server::{events::TickEvent, Server, UserKey};

use logging::info;

use world_server_naia_proto::{
    behavior as shared_behavior,
    channels::PlayerCommandChannel,
    components::{LookDirection, NextTilePosition, PhysicsController, TileMovementType},
    messages::PlayerCommands,
};

use world_server_naia_proto::components::NetworkedMoveBuffer;

use crate::{
    asset::AssetManager,
    user::{components::ServerTileMovement, UserManager},
};

#[derive(Resource)]
struct CachedTickEventsState {
    event_state: SystemState<EventReader<'static, 'static, TickEvent>>,
}

pub fn tick_events_startup(world: &mut World) {
    let event_state: SystemState<EventReader<TickEvent>> = SystemState::new(world);
    world.insert_resource(CachedTickEventsState { event_state });
}

pub fn tick_events(world: &mut World) {
    let mut tick_events = Vec::new();
    world.resource_scope(
        |world, mut events_reader_state: Mut<CachedTickEventsState>| {
            let mut events_reader = events_reader_state.event_state.get_mut(world);

            for TickEvent(server_tick) in events_reader.read() {
                tick_events.push(*server_tick);
            }
        },
    );

    if tick_events.is_empty() {
        return;
    }

    {
        let mut system_state: SystemState<(
            Server,
            Res<UserManager>,
            Query<(Entity, &mut ServerTileMovement, &mut PhysicsController)>,
            Query<&mut NextTilePosition>,
            Query<&mut NetworkedMoveBuffer>,
            Query<&mut LookDirection>,
        )> = SystemState::new(world);
        let (
            mut server,
            user_manager,
            mut tile_movement_q,
            mut next_tile_position_q,
            mut net_move_buffer_q,
            mut lookdir_q,
        ) = system_state.get_mut(world);

        for server_tick in tick_events.iter() {
            let mut users_without_command = user_manager.user_key_set();

            // receive & process command messages
            let mut user_commands: HashMap<UserKey, Option<PlayerCommands>> = HashMap::new();
            let mut messages = server.receive_tick_buffer_messages(server_tick);
            for (user_key, incoming_command) in
                messages.read::<PlayerCommandChannel, PlayerCommands>()
            {
                users_without_command.remove(&user_key);

                if let Some(prev_command) = user_commands.get_mut(&user_key) {
                    if let Some(prev_command_2) = prev_command.as_mut() {
                        prev_command_2.merge_newer(&incoming_command);
                    } else {
                        *prev_command = Some(incoming_command);
                    }
                } else {
                    user_commands.insert(user_key, Some(incoming_command));
                }
            }

            // process null commands
            for user_key in users_without_command {
                if user_commands.contains_key(&user_key) {
                    panic!("User should not have a command");
                } else {
                    user_commands.insert(user_key, None);
                }
            }

            // All game logic should happen here, on a tick event

            // process movement
            for (entity, mut tile_movement, mut physics) in tile_movement_q.iter_mut() {
                let Some(user_key) = user_manager.get_user_key_from_entity(&entity) else {
                    continue;
                };

                let Some(player_command) = user_commands.remove(&user_key) else {
                    panic!("No command found for user: {:?}", user_key);
                    // multiple commands per entity??
                };
                let Ok(mut look_dir) = lookdir_q.get_mut(entity) else {
                    panic!("LookDirection not found for entity: {:?}", entity);
                };

                // if let Some(player_commands) = player_command.as_ref() {
                //     if let Some(move_dir) = player_commands.get_move() {
                //         // let distance = tile_movement.get_dis();
                //         // info!("Recv Move Command. Tick: {:?}. MoveDir: {:?}. Dis: {:?}", server_tick, move_dir, distance);
                //     } else {
                //         info!("Recv Move Command. Tick: {:?}. MoveDir: None", server_tick);
                //     }
                // }

                let (inner_tile_movement, inner_move_buffer) = tile_movement.decompose();
                let (result, mut ntp_output, mut nmb_output) = shared_behavior::process_tick(
                    TileMovementType::Server,
                    *server_tick,
                    player_command,
                    inner_tile_movement,
                    &mut physics,
                    inner_move_buffer,
                    Some(&mut look_dir),
                );
                let (ntp_result, nmb_result) = shared_behavior::process_result(
                    inner_tile_movement,
                    inner_move_buffer,
                    &mut physics,
                    result,
                );
                if ntp_result.is_some() {
                    ntp_output = ntp_result;
                }
                if nmb_result.is_some() {
                    nmb_output = nmb_result;
                }

                if let Some((outbound_tile_x, outbound_tile_y)) = ntp_output {
                    // send updates
                    let outbound_velocity = physics.velocity();
                    let Ok(mut next_tile_position) = next_tile_position_q.get_mut(entity) else {
                        panic!("NextTilePosition not found for entity: {:?}", entity);
                    };
                    tile_movement.send_updated_next_tile_position(
                        *server_tick,
                        &mut next_tile_position,
                        outbound_tile_x,
                        outbound_tile_y,
                        outbound_velocity.x,
                        outbound_velocity.y,
                    );
                }
                if let Some(nmb) = nmb_output {
                    // send updates
                    let Ok(mut net_move_buffer) = net_move_buffer_q.get_mut(entity) else {
                        panic!("NetworkedMoveBuffer not found for entity: {:?}", entity);
                    };
                    tile_movement.send_updated_net_move_buffer(
                        &physics,
                        *server_tick,
                        &mut net_move_buffer,
                        nmb,
                    );
                }
            }
        }
    }

    if !tick_events.is_empty() {
        handle_scope_checks(world);
    }
}

fn handle_scope_checks(world: &mut World) {
    // Asset scope checks
    // TODO: this does not belong here... see notes

    // 1. get all scope checks from server
    let mut scope_checks = Vec::new();

    {
        let mut system_state: SystemState<Server> = SystemState::new(world);
        let server = system_state.get_mut(world);

        // Update scopes of entities
        for (room_key, user_key, entity) in server.scope_checks() {
            let in_scope = server.user_scope(&user_key).has(&entity);
            scope_checks.push((room_key, user_key, entity, in_scope));
        }
    }

    if scope_checks.is_empty() {
        return;
    }

    // 2. calculate all updates to scope needed
    let mut scope_actions: HashMap<(UserKey, Entity), bool> = HashMap::new();

    for (_room_key, user_key, entity, in_scope) in scope_checks {
        // TODO: assess scope logic here ..
        // right now, everything not in scope is added to user
        // however this will change later

        if !in_scope {
            info!(
                "Entity out of scope: {:?}, should be added to user.",
                entity
            );
            scope_actions.insert((user_key, entity), true);
        }
    }

    if scope_actions.is_empty() {
        return;
    }

    // 3. actually update all scopes
    {
        let mut system_state: SystemState<Server> = SystemState::new(world);
        let mut server = system_state.get_mut(world);

        for ((user_key, entity), include) in scope_actions.iter() {
            if *include {
                if server.user_scope(&user_key).has(&entity) {
                    panic!("Entity already in scope, shouldn't happen");
                }
                info!("Adding entity to user scope: {:?}", entity);
                server.user_scope_mut(&user_key).include(&entity);
            } else {
                if !server.user_scope(&user_key).has(&entity) {
                    panic!("Entity already out of scope, shouldn't happen");
                }
                info!("Removing entity from user scope: {:?}", entity);
                server.user_scope_mut(&user_key).exclude(&entity);
            }
        }
    }

    AssetManager::handle_asset_ref_scope_events(world, scope_actions);
}
