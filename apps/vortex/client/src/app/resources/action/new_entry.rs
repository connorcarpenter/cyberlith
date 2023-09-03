use bevy_ecs::{
    prelude::{Commands, Entity, Query, World},
    system::{Res, ResMut, SystemState},
};

use naia_bevy_client::Client;

use render_api::components::Visibility;

use vortex_proto::{
    components::{ChangelistEntry, EntryKind},
    FileExtension,
};

use crate::app::{
    components::{
        file_system::{ChangelistUiState, FileSystemParent, FileSystemUiState},
        OwnedByFileLocal,
    },
    resources::{
        toolbar::Toolbar,
        action::Action, action_stack::ActionStack, camera_manager::CameraManager, canvas::Canvas,
        file_tree::FileTree, global::Global, shape_manager::ShapeManager, tab_manager::TabManager,
    },
};

pub(crate) fn execute(
    world: &mut World,
    action_stack: &mut ActionStack,
    parent_entity_opt: Option<Entity>,
    new_file_name: String,
    entry_kind: EntryKind,
    old_entity_opt: Option<Entity>,
    entry_contents_opt: Option<Vec<FileTree>>,
) -> Vec<Action> {
    let mut system_state: SystemState<(
        Commands,
        Client,
        Res<Global>,
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
        global,
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
        ActionStack::deselect_all_selected_files(&mut client, &mut fs_query, &mut cl_query);
    ActionStack::release_entities(&mut commands, &mut client, file_entries_to_release);

    let parent_entity = {
        if let Some(parent_entity) = parent_entity_opt {
            parent_entity
        } else {
            global.project_root_entity
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

    let entity_id = action_stack.create_fs_entry(
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

    system_state.apply(world);

    return vec![Action::DeleteEntry(
        entity_id,
        Some(deselected_row_entities),
    )];
}
