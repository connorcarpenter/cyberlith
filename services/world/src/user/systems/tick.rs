use std::collections::HashMap;

use bevy_ecs::{
    change_detection::{Mut, ResMut},
    entity::Entity,
    event::EventReader,
    prelude::{Query, Resource, World},
    system::{Res, SystemState},
};

use naia_bevy_server::{
    events::TickEvent,
    Server, UserKey,
};

use bevy_http_client::HttpClient;
use logging::info;
use world_server_naia_proto::{
    components::{Position, Alt1, AssetEntry, AssetRef, Main},
    channels::PlayerCommandChannel,
    messages::KeyCommand,
    behavior as shared_behavior,
};

use crate::{world_instance::WorldInstance, asset::AssetManager, user::UserManager};

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
        let mut system_state: SystemState<(Server, Query<&mut Position>)> = SystemState::new(world);
        let (mut server, mut position_q) = system_state.get_mut(world);

        for server_tick in tick_events.iter() {
            // All game logic should happen here, on a tick event

            let mut messages = server.receive_tick_buffer_messages(server_tick);
            for (_user_key, key_command) in messages.read::<PlayerCommandChannel, KeyCommand>() {
                let Some(entity) = &key_command.entity.get(&server) else {
                    continue;
                };
                let Ok(mut position) = position_q.get_mut(*entity) else {
                    continue;
                };
                shared_behavior::process_command(&key_command, &mut position);
            }
        }
    }

    // Asset scope checks
    // TODO: this does not belong here... see notes

    // get all scope checks from server
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

    // calculate all updates to scope needed
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

    // actually update all scopes
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

    // determine if any entities that have gone into or out of scope have AssetRef components
    let mut asset_id_entity_actions = Vec::new();

    {
        let mut system_state: SystemState<(
            Server,
            Query<&AssetEntry>,
            Query<&AssetRef<Main>>,
            Query<&AssetRef<Alt1>>,
        )> = SystemState::new(world);
        let (server, asset_entry_q, asset_ref_main_q, asset_ref_alt1_q) =
            system_state.get_mut(world);

        for ((user_key, entity), include) in scope_actions.iter() {
            // determine if entity has any AssetRef components
            info!("Checking entity for AssetRefs: {:?}", entity);

            // AssetRef<Main>
            if let Ok(asset_ref) = asset_ref_main_q.get(*entity) {
                let asset_id_entity = asset_ref.asset_id_entity.get(&server).unwrap();
                let asset_id = *asset_entry_q.get(asset_id_entity).unwrap().asset_id;

                info!(
                    "entity {:?} has AssetRef<Main>(asset_id: {:?})",
                    entity, asset_id
                );

                asset_id_entity_actions.push((*user_key, asset_id, *include));
            }
            // AssetRef<Alt1>
            if let Ok(asset_ref) = asset_ref_alt1_q.get(*entity) {
                let asset_id_entity = asset_ref.asset_id_entity.get(&server).unwrap();
                let asset_id = *asset_entry_q.get(asset_id_entity).unwrap().asset_id;

                info!(
                    "entity {:?} has AssetRef<Alt1>(asset_id: {:?})",
                    entity, asset_id
                );

                asset_id_entity_actions.push((*user_key, asset_id, *include));
            }
            // this is unecessary, just for logging
            if let Ok(asset_entry) = asset_entry_q.get(*entity) {
                let asset_id = *asset_entry.asset_id;

                info!(
                    "entity {:?} has AssetEntry(asset_id: {:?})",
                    entity, asset_id
                );
            }

            // TODO: put other AssetRef<Marker> components here .. also could clean this up somehow??
        }
    }

    if asset_id_entity_actions.is_empty() {
        return;
    }

    // update asset id entities in asset manager
    {
        let mut system_state: SystemState<(
            Server,
            Res<WorldInstance>,
            Res<UserManager>,
            ResMut<AssetManager>,
            ResMut<HttpClient>,
        )> = SystemState::new(world);
        let (
            mut server,
            world_instance,
            user_manager,
            mut asset_manager,
            mut http_client
        ) = system_state.get_mut(world);

        asset_manager.handle_scope_actions(
            &mut server,
            &world_instance,
            &user_manager,
            &mut http_client,
            asset_id_entity_actions,
        );
    }
}
