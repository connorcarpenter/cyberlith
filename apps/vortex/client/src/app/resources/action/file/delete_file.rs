use bevy_ecs::{
    prelude::{Commands, Entity, Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::components::{
    ChangelistEntry, EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild,
};

use crate::app::{
    components::file_system::{ChangelistUiState, FileSystemParent, FileSystemUiState},
    resources::{
        action::file::{
            select_file::{request_entities, select_files},
            FileAction,
        },
        file_manager::FileManager,
        file_tree::FileTree,
        tab_manager::TabManager,
    },
};

pub(crate) fn execute(
    world: &mut World,
    project_root_entity: Entity,
    action: FileAction,
) -> Vec<FileAction> {
    let FileAction::DeleteFile(file_entity, files_to_select_opt) = action else {
        panic!("Expected DeleteFile");
    };

    info!("DeleteFile({:?})", file_entity);
    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<FileManager>,
        ResMut<TabManager>,
        Query<(Entity, &mut FileSystemUiState)>,
        Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
        Query<(
            &FileSystemEntry,
            Option<&FileSystemChild>,
            Option<&FileSystemRootChild>,
        )>,
        Query<&mut FileSystemParent>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut file_manager,
        mut tab_manager,
        mut ui_q,
        mut cl_q,
        fs_query,
        mut parent_q,
    ) = system_state.get_mut(world);
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
        parent_q
            .get_mut(parent_entity)
            .unwrap()
            .remove_child(&file_entity);

        Some(parent_entity)
    } else if let Some(_) = fs_root_child_opt {
        // remove entity from root
        parent_q
            .get_mut(project_root_entity)
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
                let entries =
                    convert_contents_to_slim_tree(&client, &file_entity, &fs_query, &mut parent_q);

                Some(entries)
            }
        }
    };

    // actually delete the entry
    commands.entity(file_entity).despawn();

    // select files as needed
    if let Some(files_to_select) = files_to_select_opt {
        let file_entries_to_request =
            select_files(&mut client, &mut ui_q, &mut cl_q, &files_to_select);
        request_entities(&mut commands, &mut client, file_entries_to_request);
    }

    // file manager track deleted file
    file_manager.on_file_delete(&mut client, &mut tab_manager, &file_entity);

    system_state.apply(world);

    return vec![FileAction::CreateFile(
        parent_entity_opt,
        entry_name,
        entry_kind,
        Some(file_entity),
        entry_contents_opt.map(|entries| entries.into_iter().map(|(_, tree)| tree).collect()),
    )];
}

pub(crate) fn convert_contents_to_slim_tree(
    client: &Client,
    parent_entity: &Entity,
    fs_query: &Query<(
        &FileSystemEntry,
        Option<&FileSystemChild>,
        Option<&FileSystemRootChild>,
    )>,
    parent_query: &mut Query<&mut FileSystemParent>,
) -> Vec<(Entity, FileTree)> {
    let mut trees = Vec::new();

    if let Ok(parent) = parent_query.get(*parent_entity) {
        let children_entities = parent.get_children();
        for child_entity in children_entities {
            let (child_entry, _, _) = fs_query.get(child_entity).unwrap();
            let slim_tree = FileTree::new(
                child_entity,
                child_entry.name.to_string(),
                *child_entry.kind,
            );
            trees.push((child_entity, slim_tree));
        }

        for (entry_entity, tree) in trees.iter_mut() {
            let subtree =
                convert_contents_to_slim_tree(client, entry_entity, fs_query, parent_query);
            if subtree.len() > 0 {
                tree.children = Some(
                    subtree
                        .into_iter()
                        .map(|(_, child_tree)| child_tree)
                        .collect(),
                );
            }
        }
    }

    trees
}
