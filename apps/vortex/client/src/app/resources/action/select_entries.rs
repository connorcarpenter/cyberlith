
use bevy_ecs::{entity::Entity, system::{Commands, Query, SystemState}, world::World};

use naia_bevy_client::Client;
use vortex_proto::components::ChangelistEntry;

use crate::app::{resources::{action::Action, action_stack::ActionStack}, components::file_system::{ChangelistUiState, FileSystemUiState}};

pub(crate) fn execute(world: &mut World, file_entities: Vec<Entity>) -> Vec<Action> {
    let mut system_state: SystemState<(
        Commands,
        Client,
        Query<(Entity, &mut FileSystemUiState)>,
        Query<(Entity, &ChangelistEntry, &mut ChangelistUiState)>,
    )> = SystemState::new(world);
    let (mut commands, mut client, mut fs_query, mut cl_query) =
        system_state.get_mut(world);

    // TODO: when shift/control is pressed, select multiple items

    // Deselect all selected files, select the new selected files
    let (deselected_row_entities, mut file_entries_to_release) =
        ActionStack::deselect_all_selected_files(&mut client, &mut fs_query, &mut cl_query);
    let mut file_entries_to_request =
        ActionStack::select_files(&mut client, &mut fs_query, &mut cl_query, &file_entities);

    ActionStack::remove_duplicates(&mut file_entries_to_release, &mut file_entries_to_request);

    ActionStack::release_entities(&mut commands, &mut client, file_entries_to_release);
    ActionStack::request_entities(&mut commands, &mut client, file_entries_to_request);

    system_state.apply(world);

    return vec![Action::SelectEntries(deselected_row_entities)];
}