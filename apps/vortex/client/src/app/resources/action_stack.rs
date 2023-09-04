
use bevy_ecs::{
    prelude::{Commands, Entity, Resource, World},
    system::SystemState,
};

use naia_bevy_client::{Client, CommandsExt, EntityAuthStatus};

use crate::app::resources::{
    action::Action,
    shape_manager::{CanvasShape, ShapeManager},
};

#[derive(Resource)]
pub struct ActionStack {
    buffered_actions: Vec<Action>,
    undo_actions: Vec<Action>,
    redo_actions: Vec<Action>,
    undo_enabled: bool,
    redo_enabled: bool,
    buffered_check: bool,
}

impl Default for ActionStack {
    fn default() -> Self {
        Self {
            buffered_actions: Vec::new(),
            undo_actions: Vec::new(),
            redo_actions: Vec::new(),
            undo_enabled: true,
            redo_enabled: true,
            buffered_check: false,
        }
    }
}

impl ActionStack {
    pub fn buffer_action(&mut self, action: Action) {
        self.buffered_actions.push(action);
    }

    pub fn has_undo(&self) -> bool {
        !self.undo_actions.is_empty() && self.undo_enabled
    }

    pub fn has_redo(&self) -> bool {
        !self.redo_actions.is_empty() && self.redo_enabled
    }

    pub fn undo_action(&mut self, world: &mut World) {
        if !self.undo_enabled {
            panic!("Undo is disabled!");
        }
        let Some(action) = self.undo_actions.pop() else {
            panic!("No executed actions to undo!");
        };

        let mut reversed_actions = self.execute_action(world, action);

        self.redo_actions.append(&mut reversed_actions);

        self.enable_top(world);
    }

    pub fn redo_action(&mut self, world: &mut World) {
        if !self.redo_enabled {
            panic!("Redo is disabled!");
        }
        let Some(action) = self.redo_actions.pop() else {
            panic!("No undone actions to redo!");
        };

        let mut reversed_actions = self.execute_action(world, action);

        self.undo_actions.append(&mut reversed_actions);

        self.enable_top(world);
    }

    pub fn execute_actions(&mut self, world: &mut World) {
        if self.buffered_check {
            self.enable_top(world);
            self.buffered_check = false;
        }
        if self.buffered_actions.is_empty() {
            return;
        }
        let drained_actions: Vec<Action> = self.buffered_actions.drain(..).collect();
        for action in drained_actions {
            let mut reversed_actions = self.execute_action(world, action);
            self.undo_actions.append(&mut reversed_actions);
        }
        self.redo_actions.clear();

        self.enable_top(world);
    }

    fn execute_action(&mut self, world: &mut World, action: Action) -> Vec<Action> {
        action.execute(world, self)
    }

    pub fn entity_update_auth_status(&mut self, shape_manager: &mut ShapeManager, entity: &Entity) {
        // if either the undo or redo stack's top entity is this entity, then we need to enable/disable undo based on new auth status
        Self::entity_update_auth_status_impl(
            shape_manager,
            &mut self.buffered_check,
            self.undo_actions.last(),
            entity,
        );
        Self::entity_update_auth_status_impl(
            shape_manager,
            &mut self.buffered_check,
            self.redo_actions.last(),
            entity,
        );
    }

    fn entity_update_auth_status_impl(
        shape_manager: &mut ShapeManager,
        buffered_check: &mut bool,
        action_opt: Option<&Action>,
        entity: &Entity,
    ) {
        match action_opt {
            Some(Action::SelectEntries(file_entities)) => {
                if file_entities.contains(entity) {
                    *buffered_check = true;
                }
            }
            Some(Action::SelectShape(vertex_2d_entity_opt)) => {
                if let Some((vertex_2d_entity, CanvasShape::Vertex)) = vertex_2d_entity_opt {
                    let vertex_3d_entity = shape_manager
                        .vertex_entity_2d_to_3d(vertex_2d_entity)
                        .unwrap();
                    if vertex_3d_entity == *entity {
                        *buffered_check = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn enable_top(&mut self, world: &mut World) {
        Self::enable_top_impl(world, self.undo_actions.last(), &mut self.undo_enabled);
        Self::enable_top_impl(world, self.redo_actions.last(), &mut self.redo_enabled);
    }

    fn enable_top_impl(world: &mut World, last_action: Option<&Action>, enabled: &mut bool) {
        match last_action {
            Some(Action::SelectEntries(entities)) => {
                *enabled = Self::should_be_enabled(world, entities);
            }
            Some(Action::SelectShape(vertex_2d_entity_opt)) => {
                let mut entities = Vec::new();

                if let Some((vertex_2d_entity, CanvasShape::Vertex)) = vertex_2d_entity_opt {
                    let vertex_3d_entity = world
                        .get_resource::<ShapeManager>()
                        .unwrap()
                        .vertex_entity_2d_to_3d(vertex_2d_entity)
                        .unwrap();
                    entities.push(vertex_3d_entity);
                }

                *enabled = Self::should_be_enabled(world, &entities);
            }
            _ => {
                *enabled = true;
            }
        }
    }

    fn should_be_enabled(world: &mut World, entities: &Vec<Entity>) -> bool {
        let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
        let (mut commands, client) = system_state.get_mut(world);

        for file_entity in entities {
            if let Some(EntityAuthStatus::Available) =
                commands.entity(*file_entity).authority(&client)
            {
                // enabled should continue being true
            } else {
                return false;
            }
        }
        return true;
    }

    pub(crate) fn migrate_file_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        for action_list in [&mut self.undo_actions, &mut self.redo_actions] {
            for action in action_list.iter_mut() {
                action.migrate_file_entities(old_entity, new_entity);
            }
        }
    }

    pub(crate) fn migrate_vertex_entities(
        &mut self,
        old_2d_entity: Entity,
        new_2d_entity: Entity,
        old_3d_entity: Entity,
        new_3d_entity: Entity,
    ) {
        for action_list in [&mut self.undo_actions, &mut self.redo_actions] {
            for action in action_list.iter_mut() {
                action.migrate_vertex_entities(
                    old_2d_entity,
                    new_2d_entity,
                    old_3d_entity,
                    new_3d_entity,
                );
            }
        }
    }

    pub(crate) fn migrate_edge_entities(&mut self, old_2d_entity: Entity, new_2d_entity: Entity) {
        for action_list in [&mut self.undo_actions, &mut self.redo_actions] {
            for action in action_list.iter_mut() {
                action.migrate_edge_entities(old_2d_entity, new_2d_entity);
            }
        }
    }

    pub(crate) fn migrate_face_entities(&mut self, old_2d_entity: Entity, new_2d_entity: Entity) {
        for action_list in [&mut self.undo_actions, &mut self.redo_actions] {
            for action in action_list.iter_mut() {
                action.migrate_face_entities(old_2d_entity, new_2d_entity);
            }
        }
    }
}
