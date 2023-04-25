use bevy_ecs::prelude::{Component, Entity};

// FileSystemParent
#[derive(Component)]
pub struct FileSystemParent {
    children_ids: Vec<Entity>,
}

impl FileSystemParent {
    pub fn new() -> Self {
        Self {
            children_ids: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child_id: Entity) {
        self.children_ids.push(child_id);
    }

    // pub fn remove_child(&mut self, child_id: Entity) {
    //     self.children_ids.retain(|&id| id != child_id);
    // }
    //
    // pub fn has_children(&self) -> bool {
    //     !self.children_ids.is_empty()
    // }

    pub fn get_children(&self) -> &Vec<Entity> {
        &self.children_ids
    }
}

// FileSystemUiState
#[derive(Component)]
pub struct FileSystemUiState {
    pub selected: bool,
    pub opened: bool,
}

impl FileSystemUiState {
    pub fn new() -> Self {
        Self {
            selected: false,
            opened: false,
        }
    }
}
