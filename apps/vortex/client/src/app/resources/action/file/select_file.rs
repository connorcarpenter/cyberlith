use std::collections::HashSet;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, SystemState},
    world::World,
};

use naia_bevy_client::{Client, CommandsExt};

use vortex_proto::components::{ChangelistEntry, ChangelistStatus};

use crate::app::{
    components::file_system::{ChangelistUiState, FileSystemUiState},
    resources::action::file::FileAction, plugin::Main
};

pub fn execute(world: &mut World, action: FileAction) -> Vec<FileAction> {
    let FileAction::SelectFile(file_entities) = action else {
        panic!("Expected SelectFile");
    };

    let mut system_state: SystemState<(
        Commands,
        Client<Main>,
        Query<(Entity, &mut FileSystemUiState)>,
        Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
    )> = SystemState::new(world);
    let (mut commands, mut client, mut fs_query, mut cl_query) = system_state.get_mut(world);

    // TODO: when shift/control is pressed, select multiple items

    // Deselect all selected files, select the new selected files
    let (deselected_row_entities, mut file_entries_to_release) =
        deselect_all_selected_files(&mut client, &mut fs_query, &mut cl_query);
    let mut file_entries_to_request =
        select_files(&mut client, &mut fs_query, &mut cl_query, &file_entities);

    remove_duplicates(&mut file_entries_to_release, &mut file_entries_to_request);

    release_entities(&mut commands, &mut client, file_entries_to_release);
    request_entities(&mut commands, &mut client, file_entries_to_request);

    system_state.apply(world);

    return vec![FileAction::SelectFile(deselected_row_entities)];
}

pub fn select_files(
    client: &mut Client<Main>,
    fs_query: &mut Query<(Entity, &mut FileSystemUiState)>,
    cl_query: &mut Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
    row_entities: &Vec<Entity>,
) -> HashSet<Entity> {
    let mut file_entries_to_request = HashSet::new();
    for row_entity in row_entities {
        if let Ok((_, mut ui_state)) = fs_query.get_mut(*row_entity) {
            // File System
            ui_state.selected = true;

            file_entries_to_request.insert(*row_entity);
        }
        if let Ok((_, cl_entry, mut ui_state)) = cl_query.get_mut(*row_entity) {
            // Changelist
            ui_state.selected = true;

            if *cl_entry.status != ChangelistStatus::Deleted {
                if let Some(file_entity) = cl_entry.file_entity.get(client) {
                    file_entries_to_request.insert(file_entity);
                }
            }
        }
    }
    file_entries_to_request
}

pub fn deselect_all_selected_files(
    client: &mut Client<Main>,
    fs_query: &mut Query<(Entity, &mut FileSystemUiState)>,
    cl_query: &mut Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
) -> (Vec<Entity>, HashSet<Entity>) {
    let mut deselected_row_entities = Vec::new();
    let mut file_entries_to_release = HashSet::new();
    for (item_entity, mut ui_state) in fs_query.iter_mut() {
        // FileSystem
        if ui_state.selected {
            ui_state.selected = false;

            deselected_row_entities.push(item_entity);
            file_entries_to_release.insert(item_entity);
        }
    }
    for (item_entity, cl_entry, mut ui_state) in cl_query.iter_mut() {
        // Changelist
        if ui_state.selected {
            ui_state.selected = false;

            deselected_row_entities.push(item_entity);

            if *cl_entry.status != ChangelistStatus::Deleted {
                if let Some(file_entity) = cl_entry.file_entity.get(client) {
                    file_entries_to_release.insert(file_entity);
                }
            }
        }
    }
    (deselected_row_entities, file_entries_to_release)
}

pub fn request_entities(
    commands: &mut Commands,
    client: &mut Client<Main>,
    entities_to_request: HashSet<Entity>,
) {
    for file_entity in entities_to_request {
        //info!("request_entities({:?})", file_entity);
        let mut entity_mut = commands.entity(file_entity);
        if entity_mut.authority(client).is_some() {
            entity_mut.request_authority(client);
        }
    }
}

pub fn release_entities(
    commands: &mut Commands,
    client: &mut Client<Main>,
    entities_to_release: HashSet<Entity>,
) {
    for file_entity in entities_to_release {
        let mut entity_mut = commands.entity(file_entity);
        if entity_mut.authority(client).is_some() {
            entity_mut.release_authority(client);
        }
    }
}

pub fn remove_duplicates(set_a: &mut HashSet<Entity>, set_b: &mut HashSet<Entity>) {
    set_a.retain(|item| {
        if set_b.contains(item) {
            set_b.remove(item);
            false // Remove the item from set_a
        } else {
            true // Keep the item in set_a
        }
    });
}
