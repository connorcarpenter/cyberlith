use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::SystemState,
    world::Mut,
};

use naia_bevy_client::{Client, CommandsExt, EntityAuthStatus};

use crate::app::resources::{
    action::{palette::PaletteAction, animation::AnimAction, file::{FileAction, FileActions}, shape::ShapeAction},
    canvas::Canvas,
    file_manager::FileManager,
    input_manager::InputManager,
    tab_manager::TabManager,
};

pub trait Action: Clone {
    fn entity_update_auth_status_impl(
        buffered_check: &mut bool,
        action_opt: Option<&Self>,
        entity: &Entity,
    );
    fn enable_top_impl(world: &mut World, last_action: Option<&Self>, enabled: &mut bool);
}

pub struct ActionStack<A> {
    undo_actions: Vec<A>,
    redo_actions: Vec<A>,
    undo_enabled: bool,
    redo_enabled: bool,
    buffered_check: bool,
}

impl<A> Default for ActionStack<A> {
    fn default() -> Self {
        Self {
            undo_actions: Vec::new(),
            redo_actions: Vec::new(),
            undo_enabled: true,
            redo_enabled: true,
            buffered_check: false,
        }
    }
}

pub(crate) fn action_stack_undo(world: &mut World) {
    let Some(canvas) = world.get_resource::<Canvas>() else {
        return;
    };

    let canvas_has_focus = canvas.has_focus();

    if canvas_has_focus {
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            let Some(tab_file_entity) = tab_manager.current_tab_entity() else {
                return;
            };
            let tab_file_entity = *tab_file_entity;
            if let Some(tab_state) = tab_manager.current_tab_state_mut() {
                if tab_state.action_stack.has_undo() {
                    world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
                        tab_state.action_stack.undo_action(
                            world,
                            &mut input_manager,
                            tab_file_entity,
                        );
                    });
                }
            }
        });
    } else {
        world.resource_scope(|world, mut file_actions: Mut<FileActions>| {
            let file_manager = world.get_resource::<FileManager>().unwrap();

            let project_root_entity = file_manager.project_root_entity;

            if file_actions.has_undo() {
                file_actions.undo_action(world, project_root_entity);
            }
        });
    }
}

pub(crate) fn action_stack_redo(world: &mut World) {
    let Some(canvas) = world.get_resource::<Canvas>() else {
        return;
    };

    let canvas_has_focus = canvas.has_focus();

    if canvas_has_focus {
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            let Some(tab_file_entity) = tab_manager.current_tab_entity() else {
                return;
            };
            let tab_file_entity = *tab_file_entity;
            if let Some(tab_state) = tab_manager.current_tab_state_mut() {
                if tab_state.action_stack.has_redo() {
                    world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
                        tab_state.action_stack.redo_action(
                            world,
                            &mut input_manager,
                            tab_file_entity,
                        );
                    });
                }
            }
        });
    } else {
        world.resource_scope(|world, mut file_actions: Mut<FileActions>| {
            let file_manager = world.get_resource::<FileManager>().unwrap();
            let project_root_entity = file_manager.project_root_entity;

            if file_actions.has_redo() {
                file_actions.redo_action(world, project_root_entity);
            }
        });
    }
}

impl<A: Action> ActionStack<A> {
    pub fn has_undo(&self) -> bool {
        !self.undo_actions.is_empty() && self.undo_enabled
    }

    pub fn has_redo(&self) -> bool {
        !self.redo_actions.is_empty() && self.redo_enabled
    }

    pub fn pop_undo(&mut self) -> A {
        if !self.undo_enabled {
            panic!("Undo is disabled!");
        }
        let Some(action) = self.undo_actions.pop() else {
            panic!("No executed actions to undo!");
        };

        action
    }

    pub fn post_execute_undo(&mut self, world: &mut World, mut reversed_actions: Vec<A>) {
        self.redo_actions.append(&mut reversed_actions);

        self.enable_top(world);
    }

    pub fn pop_redo(&mut self) -> A {
        if !self.redo_enabled {
            panic!("Redo is disabled!");
        }
        let Some(action) = self.redo_actions.pop() else {
            panic!("No undone actions to redo!");
        };

        action
    }

    pub fn post_execute_redo(&mut self, world: &mut World, mut reversed_actions: Vec<A>) {
        self.undo_actions.append(&mut reversed_actions);

        self.enable_top(world);
    }

    pub fn check_top(&mut self, world: &mut World) {
        if self.buffered_check {
            self.enable_top(world);
            self.buffered_check = false;
        }
    }

    pub fn entity_update_auth_status(&mut self, entity: &Entity) {
        // if either the undo or redo stack's top entity is this entity, then we need to enable/disable undo based on new auth status
        A::entity_update_auth_status_impl(
            &mut self.buffered_check,
            self.undo_actions.last(),
            entity,
        );
        A::entity_update_auth_status_impl(
            &mut self.buffered_check,
            self.redo_actions.last(),
            entity,
        );
    }

    fn enable_top(&mut self, world: &mut World) {
        A::enable_top_impl(world, self.undo_actions.last(), &mut self.undo_enabled);
        A::enable_top_impl(world, self.redo_actions.last(), &mut self.redo_enabled);
    }

    pub fn should_be_enabled(world: &mut World, entities: &Vec<Entity>) -> bool {
        let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
        let (mut commands, client) = system_state.get_mut(world);

        for entity in entities {
            if let Some(entity_commands) = commands.get_entity(*entity) {
                if let Some(EntityAuthStatus::Available) = entity_commands.authority(&client) {
                    // enabled should continue being true
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        return true;
    }

    pub fn post_action_execution(&mut self, world: &mut World, mut reversed_actions: Vec<A>) {
        self.undo_actions.append(&mut reversed_actions);
        self.redo_actions.clear();
        self.enable_top(world);
    }
}

impl ActionStack<FileAction> {
    pub fn execute_action(
        &mut self,
        world: &mut World,
        project_root_entity: Entity,
        action: FileAction,
    ) -> Vec<FileAction> {
        action.execute(world, project_root_entity, self)
    }

    pub(crate) fn migrate_file_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        for action_list in [&mut self.undo_actions, &mut self.redo_actions] {
            for action in action_list.iter_mut() {
                action.migrate_file_entities(old_entity, new_entity);
            }
        }
    }
}

impl ActionStack<ShapeAction> {
    pub fn execute_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        tab_file_entity: Entity,
        action: ShapeAction,
    ) -> Vec<ShapeAction> {
        action.execute(world, input_manager, tab_file_entity, self)
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

impl ActionStack<AnimAction> {
    pub fn execute_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        tab_file_entity: Entity,
        action: AnimAction,
    ) -> Vec<AnimAction> {
        action.execute(world, input_manager, tab_file_entity)
    }
}

impl ActionStack<PaletteAction> {
    pub fn execute_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        tab_file_entity: Entity,
        action: PaletteAction,
    ) -> Vec<PaletteAction> {
        action.execute(world, input_manager, tab_file_entity)
    }
}