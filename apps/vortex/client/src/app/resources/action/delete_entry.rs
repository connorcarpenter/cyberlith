use bevy_ecs::{
    prelude::{Commands, Entity, Query, World},
    system::{Res, SystemState},
};

use naia_bevy_client::Client;

use vortex_proto::components::{
    ChangelistEntry, EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild,
};

use crate::app::{
    components::file_system::{ChangelistUiState, FileSystemParent, FileSystemUiState},
    resources::{action::Action, action_stack::ActionStack, global::Global},
};

pub(crate) fn execute(
    world: &mut World,
    file_entity: Entity,
    files_to_select_opt: Option<Vec<Entity>>,
) -> Vec<Action> {
    let mut system_state: SystemState<(
        Commands,
        Client,
        Res<Global>,
        Query<(Entity, &mut FileSystemUiState)>,
        Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
        Query<(
            &FileSystemEntry,
            Option<&FileSystemChild>,
            Option<&FileSystemRootChild>,
        )>,
        Query<&mut FileSystemParent>,
    )> = SystemState::new(world);
    let (mut commands, mut client, global, mut ui_query, mut cl_query, fs_query, mut parent_query) =
        system_state.get_mut(world);
    let (entry, fs_child_opt, fs_root_child_opt) = fs_query.get(file_entity).unwrap();

    // get name of file
    let entry_name = entry.name.to_string();
    let entry_kind = *entry.kind;

    // get parent entity
    let parent_entity_opt: Option<Entity> = if let Some(fs_child) = fs_child_opt {
        // get parent entity
        let Some(parent_entity) = fs_child.parent_id.get(&client) else {
            panic!("FileSystemChild {:?} has no parent!", file_entity);
        };
        // remove entity from parent
        parent_query
            .get_mut(parent_entity)
            .unwrap()
            .remove_child(&file_entity);

        Some(parent_entity)
    } else if let Some(_) = fs_root_child_opt {
        // remove entity from root
        parent_query
            .get_mut(global.project_root_entity)
            .unwrap()
            .remove_child(&file_entity);

        None
    } else {
        panic!(
            "FileSystemEntry {:?} has neither FileSystemChild nor FileSystemRootChild!",
            file_entity
        );
    };

    let entry_contents_opt = {
        match entry_kind {
            EntryKind::File => None,
            EntryKind::Directory => {
                let entries = ActionStack::convert_contents_to_slim_tree(
                    &client,
                    &file_entity,
                    &fs_query,
                    &mut parent_query,
                );

                Some(entries)
            }
        }
    };

    // actually delete the entry
    commands.entity(file_entity).despawn();

    // select files as needed
    if let Some(files_to_select) = files_to_select_opt {
        let file_entries_to_request =
            ActionStack::select_files(&mut client, &mut ui_query, &mut cl_query, &files_to_select);
        ActionStack::request_entities(&mut commands, &mut client, file_entries_to_request);
    }

    system_state.apply(world);

    return vec![Action::NewEntry(
        parent_entity_opt,
        entry_name,
        entry_kind,
        Some(file_entity),
        entry_contents_opt.map(|entries| entries.into_iter().map(|(_, tree)| tree).collect()),
    )];
}
