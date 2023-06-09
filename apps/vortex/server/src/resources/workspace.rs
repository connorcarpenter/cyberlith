use std::collections::HashMap;

use bevy_ecs::entity::Entity;

use bevy_log::info;

use naia_bevy_server::RoomKey;

use vortex_proto::{components::EntryKind, resources::{FileEntryKey, FileEntryValue}};

pub struct Workspace {
    pub room_key: RoomKey,
    pub master_file_entries: HashMap<FileEntryKey, FileEntryValue>,
    pub working_file_entries: HashMap<FileEntryKey, FileEntryValue>,
}

impl Workspace {
    pub fn new(room_key: RoomKey, file_entries: HashMap<FileEntryKey, FileEntryValue>) -> Self {
        let working_file_tree = file_entries.clone();
        Self {
            room_key,
            master_file_entries: file_entries,
            working_file_entries: working_file_tree,
        }
    }

    pub fn create_file(&mut self, name: &str, kind: EntryKind, entity: Entity, parent: Option<FileEntryKey>) {

        let file_entry_key = FileEntryKey::new_with_parent(parent.clone(), name, kind);
        let file_entry_val = FileEntryValue::new(entity, parent, None);

        // Add new Entity into Working Tree
        Self::add_to_file_tree(&mut self.working_file_entries, file_entry_key, file_entry_val);

        // Update changelist

        // check whether newly added file already exists in master tree


        // check whether a changelist entry already exists for this file

        // if file doesn't exist in master tree and no changelist entry exists, then create a changelist entry

        // if file exists in master tree and a changelist entry exists, then delete the changelist entry
    }

    pub fn delete_file(&mut self, entity: &Entity) -> Vec<Entity> {

        // Remove Entity from Working Tree, returning a list of child entities that should be despawned
        let output = Self::remove_file_entry(&mut self.working_file_entries, entity);

        // Update changelist

        // check whether newly added file already exists in master tree

        // check whether a changelist entry already exists for this file

        // if file doesn't exist in master tree and a changelist entry exists, then delete the changelist entry

        // if file exists in master tree and no changelist entry exists, then create a changelist entry

        output
    }

    fn add_to_file_tree(file_entries: &mut HashMap<FileEntryKey, FileEntryValue>, file_entry_key: FileEntryKey, file_entry_value: FileEntryValue) {

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
    fn remove_file_entry(file_entries: &mut HashMap<FileEntryKey, FileEntryValue>, entity: &Entity) -> Vec<Entity> {

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

        let removed_entry = file_entries.remove(&key).unwrap();
        if removed_entry.children().is_none() {
            return Vec::new();
        }
        let removed_entry_children = removed_entry.children().unwrap();
        return Self::collect_entities(file_entries, removed_entry_children);
    }

    fn collect_entities(file_entries: &mut HashMap<FileEntryKey, FileEntryValue>, children_keys: &Vec<FileEntryKey>) -> Vec<Entity> {
        let mut entities = Vec::new();
        for child_key in children_keys {
            let entry_val = file_entries.get(child_key).expect("entry does not exist").clone();
            entities.push(entry_val.entity());
            if let Some(children) = entry_val.children() {
                let mut children_entities = Self::collect_entities(file_entries, children);
                entities.append(&mut children_entities);
            }
        }
        return entities;
    }
}
