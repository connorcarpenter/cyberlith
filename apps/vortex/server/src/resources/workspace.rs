use bevy_ecs::entity::Entity;
use bevy_log::info;
use naia_bevy_server::RoomKey;
use vortex_proto::components::EntryKind;
use vortex_proto::resources::FileTree;

pub struct Workspace {
    pub room_key: RoomKey,
    pub master_file_tree: Vec<FileTree>,
    pub working_file_tree: Vec<FileTree>,
}

impl Workspace {
    pub fn new(room_key: RoomKey, file_tree: Vec<FileTree>) -> Self {
        let working_file_tree = file_tree.clone();
        Self {
            room_key,
            master_file_tree: file_tree,
            working_file_tree,
        }
    }

    pub fn spawn_entity(&mut self, entity: Entity, name: String, kind: EntryKind, parent: Option<Entity>) {
        let Some(parent_entity) = parent else {
            self.working_file_tree.push(FileTree::new(entity, name, kind));
            info!("Added new entity into Working FileTree");
            return;
        };
        let Some(parent_file_tree) = Self::find_file_tree_by_entity_mut(&mut self.working_file_tree, &parent_entity) else {
            panic!("parent entity not found in FileTree");
        };
        parent_file_tree.children.get_or_insert(Vec::new()).push(FileTree::new(entity, name, kind));
        info!("Added new entity into Working FileTree");
    }

    // returns a list of child entities that should be despawned
    pub fn despawn_entity(&mut self, entity: &Entity) -> Vec<Entity> {
        Self::remove_file_tree_by_entity(&mut self.working_file_tree, entity)
    }

    fn find_file_tree_by_entity_mut<'a>(children: &'a mut Vec<FileTree>, entity: &Entity) -> Option<&'a mut FileTree> {
        for child in children {
            if child.entity == *entity {
                return Some(child);
            }
            if let Some(children) = &mut child.children {
                let found_child = Self::find_file_tree_by_entity_mut(children, entity);
                if found_child.is_some() {
                    return found_child;
                }
            }
        }
        return None;
    }

    fn remove_file_tree_by_entity(file_trees: &mut Vec<FileTree>, entity: &Entity) -> Vec<Entity> {

        let mut index_found = None;
        for i in 0..file_trees.len() {
            if file_trees[i].entity == *entity {
                index_found = Some(i);
                break;
            }
        }
        let Some(index) = index_found else {
            panic!("entity not found in FileTree");
        };
        let removed_tree = file_trees.remove(index);
        if removed_tree.children.is_none() {
            return Vec::new();
        }
        let removed_tree_children = removed_tree.children.unwrap();
        return Self::collect_entities(&removed_tree_children);
    }

    fn collect_entities(file_trees: &Vec<FileTree>) -> Vec<Entity> {
        let mut entities = Vec::new();
        for file_tree in file_trees {
            entities.push(file_tree.entity);
            if let Some(children) = &file_tree.children {
                let mut children_entities = Self::collect_entities(children);
                entities.append(&mut children_entities);
            }
        }
        return entities;
    }
}
