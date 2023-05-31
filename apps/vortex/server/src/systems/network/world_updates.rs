use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventReader;
use bevy_ecs::system::{Commands, Query, Res, ResMut};
use bevy_log::{info, warn};

use naia_bevy_server::events::{
    DespawnEntityEvent, InsertComponentEvents, RemoveComponentEvents, SpawnEntityEvent,
    UpdateComponentEvents,
};
use naia_bevy_server::Server;

use vortex_proto::components::{FileSystemChild, FileSystemEntry, FileSystemRootChild};
use crate::resources::{GitManager, UserManager};

pub fn spawn_entity_events(event_reader: EventReader<SpawnEntityEvent>) {
    // unused for now
    // for SpawnEntityEvent(user_key, entity) in event_reader.iter() {
    //     info!("spawned entity");
    // }
}

pub fn despawn_entity_events(
    mut commands: Commands,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut event_reader: EventReader<DespawnEntityEvent>,
) {
    for DespawnEntityEvent(user_key, entity) in event_reader.iter() {
        info!("despawned entity");

        if let Some(user) = user_manager.user_info(user_key) {
            let mut entities_to_despawn = git_manager.workspace_mut(user.get_username()).despawn_entity(entity);

            for child_entity in entities_to_despawn {
                commands.entity(child_entity).despawn();
                info!("child entity has been despawned");
            }
        } else {
            warn!("user not found");
        }

    }
}

fn despawn_recursive(entities_to_despawn: &mut Vec<Entity>, entity: Entity, server: &Server, child_query: &Query<(Entity, &FileSystemChild)>) {
    let mut new_entities_to_despawn = Vec::new();
    for (child_entity, child) in child_query.iter() {
        if let Some(parent_entity) = child.parent_id.get(server) {
            if parent_entity == entity {
                new_entities_to_despawn.push(child_entity);
            }
        }
    }

    for child_entity in &new_entities_to_despawn {
        despawn_recursive(entities_to_despawn, *child_entity, server, child_query);
    }

    entities_to_despawn.extend(new_entities_to_despawn);
}

pub fn insert_component_events(mut event_reader: EventReader<InsertComponentEvents>) {
    for events in event_reader.iter() {
        // on FileSystemEntry Insert Event
        for (user_key, entity) in events.read::<FileSystemEntry>() {}

        // on FileSystemRootChild Insert Event
        for (user_key, entity) in events.read::<FileSystemRootChild>() {}

        // on FileSystemChild Insert Event
        for (user_key, entity) in events.read::<FileSystemChild>() {}
    }
}

pub fn remove_component_events(mut event_reader: EventReader<RemoveComponentEvents>) {
    for events in event_reader.iter() {
        for (user_key, entity, component) in events.read::<FileSystemRootChild>() {
            info!("removed FileSystemRootChild component from entity");
        }
        for (user_key, entity, component) in events.read::<FileSystemChild>() {
            info!("removed FileSystemChild component from entity");
        }
    }
}

pub fn update_component_events(mut event_reader: EventReader<UpdateComponentEvents>) {
    for events in event_reader.iter() {
        // on FileSystemEntry Update Event
        for (user_key, entity) in events.read::<FileSystemEntry>() {}
        // on FileSystemChild Update Event
        for (user_key, entity) in events.read::<FileSystemChild>() {}
    }
}
