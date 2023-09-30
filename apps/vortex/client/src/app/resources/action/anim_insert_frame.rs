use bevy_ecs::{
    prelude::World,
    system::{ResMut, SystemState, Commands},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{action::AnimAction, animation_manager::AnimationManager};

pub fn execute(world: &mut World, action: AnimAction) -> Vec<AnimAction> {
    let AnimAction::InsertFrame(file_entity, frame_index) = action else {
        panic!("Expected InsertFrame");
    };

    info!("InsertFrame({:?}, {:?})", file_entity, frame_index);

    let mut system_state: SystemState<(Commands, Client, ResMut<AnimationManager>)> = SystemState::new(world);
    let (mut commands, mut client, mut animation_manager) = system_state.get_mut(world);

    // TODO: deselect frame
    // TODO: release frame auth
    let last_frame_index = animation_manager.current_frame_index();
    let last_frame_entity = animation_manager.get_frame_entity(&file_entity, last_frame_index).unwrap();
    commands.entity(last_frame_entity).release_authority(&mut client);

    let new_frame_entity = animation_manager.framing_insert_frame(&mut commands, &mut client, file_entity, frame_index);

    animation_manager.set_current_frame_index(frame_index);

    // TODO: migrate undo/redo entities

    // auth already granted for this frame

    system_state.apply(world);

    return vec![AnimAction::DeleteFrame(file_entity, frame_index, Some(last_frame_index))];
}

//info!("creating new fs entry: `{}`", new_file_name);
//
//     let entity_id = commands
//         .spawn_empty()
//         .enable_replication(client)
//         .configure_replication(ReplicationConfig::Delegated)
//         .id();
//
//     let entry = FileSystemEntry::new(new_file_name, entry_kind);
//
//     // add FileSystemChild or FileSystemRootChild component
//     if let Some(parent_entity) = parent_entity_opt {
//         let mut child_component = FileSystemChild::new();
//         child_component.parent_id.set(client, &parent_entity);
//         commands.entity(entity_id).insert(child_component);
//     } else {
//         commands.entity(entity_id).insert(FileSystemRootChild);
//     }
//
//     // add UiState component
//     file_post_process::insert_ui_state_component(commands, entity_id, true);
//
//     if *entry.kind == EntryKind::Directory {
//         let mut entry_parent_component = FileSystemParent::new();
//
//         if let Some(entry_contents) = entry_contents_opt {
//             for sub_tree in entry_contents {
//                 let (_, new_entity) = create_fs_entry(
//                     action_stack,
//                     commands,
//                     client,
//                     file_manager,
//                     &mut entry_parent_component,
//                     Some(entity_id),
//                     &sub_tree.name,
//                     sub_tree.kind,
//                     sub_tree.children,
//                 );
//                 let old_entity = sub_tree.entity;
//                 action_stack.migrate_file_entities(old_entity, new_entity);
//             }
//         }
//
//         // add FileSystemParent component
//         commands.entity(entity_id).insert(entry_parent_component);
//     }
//
//     // add child to parent
//     file_post_process::parent_add_child_entry(parent, &entry, entity_id);
//
//     commands
//         .entity(entity_id)
//         // add FileSystemEntry component
//         .insert(entry)
//         // add FileSystemEntryLocal component
//         .insert(FileSystemEntryLocal::new(new_file_name));
//
//     // register with file manager
//     let file_ext = FileExtension::from(new_file_name);
//     file_manager.on_file_create(&entity_id, file_ext);
//
//     (file_ext, entity_id)
