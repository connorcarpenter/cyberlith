use bevy_ecs::{
    prelude::{Entity, Query, World},
    system::SystemState,
};

use vortex_proto::components::FileSystemEntry;

use crate::app::resources::action::FileAction;

pub(crate) fn execute(world: &mut World, file_entity: Entity, new_name: String) -> Vec<FileAction> {
    let mut system_state: SystemState<Query<&mut FileSystemEntry>> = SystemState::new(world);
    let mut entry_query = system_state.get_mut(world);
    let Ok(mut file_entry) = entry_query.get_mut(file_entity) else {
        panic!("Failed to get FileSystemEntry for row entity {:?}!", file_entity);
    };
    let old_name: String = file_entry.name.to_string();
    *file_entry.name = new_name.clone();

    system_state.apply(world);

    return vec![FileAction::RenameFile(file_entity, old_name)];
}
