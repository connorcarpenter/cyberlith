use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Commands};

use bevy_log::info;

use naia_bevy_server::{CommandsExt, RoomKey, Server};

use vortex_proto::{
    components::{ChangelistEntry, ChangelistStatus, EntryKind},
    resources::FileEntryKey,
};

use crate::resources::{ChangelistValue, FileEntryValue};

pub struct Workspace {
    pub room_key: RoomKey,
    pub master_file_entries: HashMap<FileEntryKey, FileEntryValue>,
    pub working_file_entries: HashMap<FileEntryKey, FileEntryValue>,
    pub changelist_entries: HashMap<FileEntryKey, ChangelistValue>,
}

impl Workspace {
    pub fn new(room_key: RoomKey, file_entries: HashMap<FileEntryKey, FileEntryValue>) -> Self {
        let working_file_tree = file_entries.clone();
        Self {
            room_key,
            master_file_entries: file_entries,
            working_file_entries: working_file_tree,
            changelist_entries: HashMap::new(),
        }
    }

    pub fn create_file(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        name: &str,
        kind: EntryKind,
        entity: Entity,
        parent: Option<FileEntryKey>,
    ) {
        let file_entry_key = FileEntryKey::new_with_parent(parent.clone(), name, kind);
        let file_entry_val = FileEntryValue::new(entity, parent, None);

        // Add new Entity into Working Tree
        Self::add_to_file_tree(
            &mut self.working_file_entries,
            file_entry_key.clone(),
            file_entry_val,
        );

        // Update changelist

        // check whether newly added file already exists in master tree
        let file_exists_in_master = self.master_file_entries.contains_key(&file_entry_key);

        // check whether a changelist entry already exists for this file
        let file_exists_in_changelist = self.changelist_entries.contains_key(&file_entry_key);

        // if file doesn't exist in master tree and no changelist entry exists, then create a changelist entry
        if !file_exists_in_master && !file_exists_in_changelist {
            let changelist_status = ChangelistStatus::Created;

            let changelist_entity = commands
                .spawn_empty()
                .enable_replication(server)
                .insert(ChangelistEntry::new(
                    file_entry_key.kind(),
                    file_entry_key.name(),
                    file_entry_key.path(),
                    changelist_status,
                ))
                .id();

            // Add entity to room
            server
                .room_mut(&self.room_key)
                .add_entity(&changelist_entity);

            let changelist_value = ChangelistValue::new(changelist_entity, changelist_status);
            self.changelist_entries
                .insert(file_entry_key.clone(), changelist_value);
        }

        // if file exists in master tree and a changelist entry exists, then delete the changelist entry
        if file_exists_in_master && file_exists_in_changelist {
            let changelist_entry = self.changelist_entries.remove(&file_entry_key).unwrap();
            commands.entity(changelist_entry.entity()).despawn();
        }
    }

    pub fn delete_file(&mut self, commands: &mut Commands, server: &mut Server, entity: &Entity) {
        // Remove Entity from Working Tree, returning a list of child entities that should be despawned
        let (file_entry_key, entities_to_delete) =
            Self::remove_file_entry(&mut self.working_file_entries, entity);

        self.update_changelist_after_despawn(commands, server, &file_entry_key);

        for (child_entity, child_key) in entities_to_delete {
            commands
                .entity(child_entity)
                .take_authority(server)
                .despawn();

            self.update_changelist_after_despawn(commands, server, &child_key);
        }
    }

    fn update_changelist_after_despawn(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        file_entry_key: &FileEntryKey,
    ) {
        // Update changelist

        // check whether newly added file already exists in master tree
        let file_exists_in_master = self.master_file_entries.contains_key(&file_entry_key);

        // check whether a changelist entry already exists for this file
        let file_exists_in_changelist = self.changelist_entries.contains_key(&file_entry_key);

        // if file doesn't exist in master tree and a changelist entry exists, then delete the changelist entry
        if !file_exists_in_master && file_exists_in_changelist {
            let changelist_entry = self.changelist_entries.remove(&file_entry_key).unwrap();
            commands.entity(changelist_entry.entity()).despawn();
        }

        // if file exists in master tree and no changelist entry exists, then create a changelist entry
        if file_exists_in_master && !file_exists_in_changelist {
            let changelist_status = ChangelistStatus::Deleted;

            let changelist_entity = commands
                .spawn_empty()
                .enable_replication(server)
                .insert(ChangelistEntry::new(
                    file_entry_key.kind(),
                    file_entry_key.name(),
                    file_entry_key.path(),
                    changelist_status,
                ))
                .id();

            // Add entity to room
            server
                .room_mut(&self.room_key)
                .add_entity(&changelist_entity);

            let changelist_value = ChangelistValue::new(changelist_entity, changelist_status);
            self.changelist_entries
                .insert(file_entry_key.clone(), changelist_value);
        }
    }

    fn add_to_file_tree(
        file_entries: &mut HashMap<FileEntryKey, FileEntryValue>,
        file_entry_key: FileEntryKey,
        file_entry_value: FileEntryValue,
    ) {
        info!("Added new entity into Working FileTree");
        file_entries.insert(file_entry_key.clone(), file_entry_value.clone());

        let Some(parent_key) = file_entry_value.parent() else {
            return;
        };
        let Some(parent_file_tree) = file_entries.get_mut(&parent_key) else {
            panic!("parent does not exist in Working FileTree!");
        };
        parent_file_tree.add_child(file_entry_key.clone());
        info!("Added child to parent entry");
    }

    // fn find_file_tree_mut_by_entity<'a>(file_entries: &'a mut HashMap<FileEntryKey, FileEntryValue>, entity: &Entity) -> Option<&'a mut FileTree> {
    //     for child in file_entries {
    //         if child.entity == *entity {
    //             return Some(child);
    //         }
    //         if let Some(children) = &mut child.children {
    //             let found_child = Self::find_file_tree_mut_by_entity(children, entity);
    //             if found_child.is_some() {
    //                 return found_child;
    //             }
    //         }
    //     }
    //     return None;
    // }
    //

    fn remove_file_entry(
        file_entries: &mut HashMap<FileEntryKey, FileEntryValue>,
        entity: &Entity,
    ) -> (FileEntryKey, Vec<(Entity, FileEntryKey)>) {
        let mut entities = Vec::new();

        let mut key_opt = None;
        for (entry_key, entry_val) in file_entries.iter() {
            if entry_val.entity() == *entity {
                key_opt = Some(entry_key.clone());
                break;
            }
        }
        if key_opt.is_none() {
            panic!("entity does not exist in Working FileTree!");
        }
        let key = key_opt.unwrap();

        // remove entry
        let removed_entry =
            Self::remove_entry_and_collect_children_entities(file_entries, &key, &mut entities);

        // remove entry from parent's children
        if let Some(parent_key) = removed_entry.parent() {
            if let Some(parent) = file_entries.get_mut(&parent_key) {
                parent.remove_child(&key);
            }
        }

        return (key, entities);
    }

    fn remove_entry_and_collect_children_entities(
        file_entries: &mut HashMap<FileEntryKey, FileEntryValue>,
        key: &FileEntryKey,
        entities: &mut Vec<(Entity, FileEntryKey)>,
    ) -> FileEntryValue {
        // remove entry
        let removed_entry = file_entries.remove(key).unwrap();

        // handle children
        if let Some(removed_entry_children) = removed_entry.children() {
            for child_key in removed_entry_children {
                let removed_entry = Self::remove_entry_and_collect_children_entities(
                    file_entries,
                    child_key,
                    entities,
                );
                entities.push((removed_entry.entity(), child_key.clone()));
            }
        }

        removed_entry
    }
}
