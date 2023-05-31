use bevy_ecs::entity::Entity;
use naia_bevy_server::RoomKey;
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

    // returns a list of child entities that should be despawned
    pub fn despawn_entity(&mut self, entity: &Entity) -> Vec<Entity> {
        Self::remove_file_tree_by_entity(&mut self.working_file_tree, entity)
    }

    fn remove_file_tree_by_entity(file_trees: &mut Vec<FileTree>, entity: &Entity) -> Vec<Entity> {

        let mut index_found = None;
        for i in 0..file_trees.len() {
            if file_trees[i].entity == *entity {
                index_found = Some(i);
                break;
            }
        }
        let removed_tree_opt = {
            if let Some(index) = index_found {
                let file_tree = file_trees.remove(index);
                Some(file_tree)
            } else {
                None
            }
        };
        let Some(removed_tree) = removed_tree_opt else {
            return Vec::new();
        };
        if removed_tree.children.is_none() {
            return Vec::new();
        }
        let removed_tree_children = removed_tree.children.unwrap();
        if removed_tree_children.len() == 0 {
            return Vec::new();
        }
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
