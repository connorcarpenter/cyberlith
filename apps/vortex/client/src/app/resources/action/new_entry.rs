use bevy_ecs::{
    prelude::{Commands, Entity, Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use render_api::components::Visibility;

use vortex_proto::{
    components::{ChangelistEntry, EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild},
    FileExtension,
};

use crate::app::{
    components::{
        file_system::{ChangelistUiState, FileSystemParent, FileSystemUiState},
        OwnedByFileLocal,
    },
    resources::{
        action::{ActionStack, FileAction, select_entries::{deselect_all_selected_files, release_entities}}, camera_manager::CameraManager, canvas::Canvas,
        file_tree::FileTree, shape_manager::ShapeManager, tab_manager::TabManager,
        toolbar::Toolbar,
    },
    systems::file_post_process,
};

pub(crate) fn execute(
    world: &mut World,
    action_stack: &mut ActionStack<FileAction>,
    project_root_entity: Entity,
    parent_entity_opt: Option<Entity>,
    new_file_name: String,
    entry_kind: EntryKind,
    old_entity_opt: Option<Entity>,
    entry_contents_opt: Option<Vec<FileTree>>,
) -> Vec<FileAction> {
    info!("CreateEntry({:?})", new_file_name);

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
        ResMut<CameraManager>,
        ResMut<ShapeManager>,
        ResMut<TabManager>,
        ResMut<Toolbar>,
        Query<(Entity, &mut FileSystemUiState)>,
        Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
        Query<&mut FileSystemParent>,
        Query<(&mut Visibility, &OwnedByFileLocal)>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut canvas,
        mut camera_manager,
        mut shape_manager,
        mut tab_manager,
        mut toolbar,
        mut fs_query,
        mut cl_query,
        mut parent_query,
        mut visibility_q,
    ) = system_state.get_mut(world);

    let (deselected_row_entities, file_entries_to_release) =
        deselect_all_selected_files(&mut client, &mut fs_query, &mut cl_query);
    release_entities(&mut commands, &mut client, file_entries_to_release);

    let parent_entity = {
        if let Some(parent_entity) = parent_entity_opt {
            parent_entity
        } else {
            project_root_entity
        }
    };

    // expand parent if it's not expanded
    {
        if let Ok((_, mut fs_ui_state)) = fs_query.get_mut(parent_entity) {
            fs_ui_state.opened = true;
        }
    }

    // actually create new entry
    let mut parent = parent_query.get_mut(parent_entity).unwrap();

    let entity_id = create_fs_entry(
        action_stack,
        &mut commands,
        &mut client,
        &mut parent,
        parent_entity_opt,
        &new_file_name,
        entry_kind,
        entry_contents_opt,
    );

    // migrate undo entities
    if let Some(old_entity) = old_entity_opt {
        action_stack.migrate_file_entities(old_entity, entity_id);
    }

    if entry_kind == EntryKind::File {
        // open tab for new entry
        tab_manager.open_tab(
            &mut client,
            &mut canvas,
            &mut camera_manager,
            &mut shape_manager,
            &mut toolbar,
            &mut visibility_q,
            &entity_id,
            FileExtension::from_file_name(&new_file_name),
        );
    }

    system_state.apply(world);

    return vec![FileAction::DeleteEntry(
        entity_id,
        Some(deselected_row_entities),
    )];
}

fn create_fs_entry(
    action_stack: &mut ActionStack<FileAction>,
    commands: &mut Commands,
    client: &mut Client,
    parent: &mut FileSystemParent,
    parent_entity_opt: Option<Entity>,
    new_file_name: &str,
    entry_kind: EntryKind,
    entry_contents_opt: Option<Vec<FileTree>>,
) -> Entity {
    info!("creating new entry: `{}`", new_file_name);

    let entity_id = commands
        .spawn_empty()
        .enable_replication(client)
        .configure_replication(ReplicationConfig::Delegated)
        .id();

    let entry = FileSystemEntry::new(new_file_name, entry_kind);

    // add FileSystemChild or FileSystemRootChild component
    if let Some(parent_entity) = parent_entity_opt {
        let mut child_component = FileSystemChild::new();
        child_component.parent_id.set(client, &parent_entity);
        commands.entity(entity_id).insert(child_component);
    } else {
        commands.entity(entity_id).insert(FileSystemRootChild);
    }

    // add UiState component
    file_post_process::insert_ui_state_component(commands, entity_id, true);

    if *entry.kind == EntryKind::Directory {
        let mut entry_parent_component = FileSystemParent::new();

        if let Some(entry_contents) = entry_contents_opt {
            for sub_tree in entry_contents {
                let new_entity = create_fs_entry(
                    action_stack,
                    commands,
                    client,
                    &mut entry_parent_component,
                    Some(entity_id),
                    &sub_tree.name,
                    sub_tree.kind,
                    sub_tree.children,
                );
                let old_entity = sub_tree.entity;
                action_stack.migrate_file_entities(old_entity, new_entity);
            }
        }

        // add FileSystemParent component
        commands.entity(entity_id).insert(entry_parent_component);
    }

    // add child to parent
    file_post_process::parent_add_child_entry(parent, &entry, entity_id);

    // add FileSystemEntry component
    commands.entity(entity_id).insert(entry);

    entity_id
}
