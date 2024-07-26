use std::collections::HashMap;

use bevy_ecs::{
    change_detection::Mut,
    entity::Entity,
    event::EventReader,
    prelude::{Query, Resource, World},
    system::SystemState,
};

use naia_bevy_server::{
    events::TickEvent,
    Server, UserKey,
};

use logging::info;
use world_server_naia_proto::{
    components::{NextTilePosition, Position, PrevTilePosition, TileMovement},
    channels::PlayerCommandChannel,
    messages::KeyCommand,
    behavior as shared_behavior,
};

use crate::asset::AssetManager;

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
        let mut system_state: SystemState<(Server, Query<(&mut PrevTilePosition, &mut NextTilePosition, &mut TileMovement, &mut Position)>)> = SystemState::new(world);
        let (mut server, mut position_q) = system_state.get_mut(world);

        for server_tick in tick_events.iter() {

            // All game logic should happen here, on a tick event

            // process movement
            for (
                mut prev_tile_position,
                next_tile_position,
                mut tile_movement,
                mut position,
            ) in position_q.iter_mut()
            {
                shared_behavior::process_movement(
                    &mut prev_tile_position,
                    next_tile_position.x(),
                    next_tile_position.y(),
                    &mut tile_movement,
                    &mut position,
                    *server_tick,
                );
            }

            let mut messages = server.receive_tick_buffer_messages(server_tick);
            for (_user_key, command) in messages.read::<PlayerCommandChannel, KeyCommand>() {
                let Some(entity) = &command.entity.get(&server) else {
                    continue;
                };
                let Ok((prev_tile_position,
                       mut next_tile_position,
                       mut tile_movement,
                       _position)) = position_q.get_mut(*entity) else {
                    continue;
                };
                shared_behavior::process_command(
                    &command,
                    &prev_tile_position,
                    &mut next_tile_position,
                    &mut tile_movement,
                );
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