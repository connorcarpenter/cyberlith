use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::SystemState,
    world::Mut,
};

use naia_bevy_client::{Client, CommandsExt, EntityAuthStatus};

use vortex_proto::components::FileExtension;

use crate::app::resources::{
    action::{AnimAction, FileAction, FileActions, ShapeAction},
    canvas::Canvas,
    file_manager::FileManager,
    tab_manager::TabManager,
};

pub trait Action: Clone {
    fn execute(
        self,
        world: &mut World,
        entity_opt: Option<&Entity>,
        action_stack: &mut ActionStack<Self>,
    ) -> Vec<Self>;
    fn entity_update_auth_status_impl(
        buffered_check: &mut bool,
        action_opt: Option<&Self>,
        entity: &Entity,
    );
    fn enable_top_impl(world: &mut World, last_action: Option<&Self>, enabled: &mut bool);
}

pub enum TabActionStack {
    Shape(ActionStack<ShapeAction>),
    Animation(ActionStack<AnimAction>),
}

impl TabActionStack {
    pub fn new(file_ext: FileExtension) -> Self {
        match file_ext {
            FileExtension::Skel | FileExtension::Mesh => Self::Shape(ActionStack::default()),
            FileExtension::Anim => Self::Animation(ActionStack::default()),
            _ => {
                panic!(
                    "TabActionStack::new() called with unsupported file extension: {:?}",
                    file_ext
                );
            }
        }
    }

    pub fn buffer_shape_action(&mut self, action: ShapeAction) {
        match self {
            Self::Shape(action_stack) => {
                action_stack.buffer_action(action);
            }
            _ => {
                panic!("buffer_shape_action() called on TabActionStack::Animation");
            }
        }
    }

    pub fn buffer_anim_action(&mut self, action: AnimAction) {
        match self {
            Self::Animation(action_stack) => {
                action_stack.buffer_action(action);
            }
            _ => {
                panic!("buffer_anim_action() called on TabActionStack::Shape");
            }
        }
    }

    pub fn has_undo(&self) -> bool {
        match self {
            Self::Shape(action_stack) => action_stack.has_undo(),
            Self::Animation(action_stack) => action_stack.has_undo(),
        }
    }

    pub fn has_redo(&self) -> bool {
        match self {
            Self::Shape(action_stack) => action_stack.has_redo(),
            Self::Animation(action_stack) => action_stack.has_redo(),
        }
    }

    pub fn undo_action(&mut self, world: &mut World, entity_opt: Option<&Entity>) {
        match self {
            Self::Shape(action_stack) => {
                action_stack.undo_action(world, entity_opt);
            }
            Self::Animation(action_stack) => {
                action_stack.undo_action(world, entity_opt);
            }
        }
    }

    pub fn redo_action(&mut self, world: &mut World, entity_opt: Option<&Entity>) {
        match self {
            Self::Shape(action_stack) => {
                action_stack.redo_action(world, entity_opt);
            }
            Self::Animation(action_stack) => {
                action_stack.redo_action(world, entity_opt);
            }
        }
    }

    pub fn execute_actions(&mut self, world: &mut World, entity_opt: Option<&Entity>) {
        match self {
            Self::Shape(action_stack) => {
                action_stack.execute_actions(world, entity_opt);
            }
            Self::Animation(action_stack) => {
                action_stack.execute_actions(world, entity_opt);
            }
        }
    }

    pub fn entity_update_auth_status(&mut self, entity: &Entity) {
        match self {
            Self::Shape(action_stack) => {
                action_stack.entity_update_auth_status(entity);
            }
            Self::Animation(action_stack) => {
                action_stack.entity_update_auth_status(entity);
            }
        }
    }
}

pub struct ActionStack<A: Action> {
    buffered_actions: Vec<A>,
    undo_actions: Vec<A>,
    redo_actions: Vec<A>,
    undo_enabled: bool,
    redo_enabled: bool,
    buffered_check: bool,
}

impl<A: Action> Default for ActionStack<A> {
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
                    tab_state
                        .action_stack
                        .undo_action(world, Some(&tab_file_entity));
                }
            }
        });
    } else {
        world.resource_scope(|world, mut file_actions: Mut<FileActions>| {
            let file_manager = world.get_resource::<FileManager>().unwrap();

            let project_root_entity = file_manager.project_root_entity;

            if file_actions.has_undo() {
                file_actions.undo_action(world, Some(&project_root_entity));
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
                    tab_state
                        .action_stack
                        .redo_action(world, Some(&tab_file_entity));
                }
            }
        });
    } else {
        world.resource_scope(|world, mut file_actions: Mut<FileActions>| {
            let file_manager = world.get_resource::<FileManager>().unwrap();
            let project_root_entity = file_manager.project_root_entity;

            if file_actions.has_redo() {
                file_actions.redo_action(world, Some(&project_root_entity));
            }
        });
    }
}

impl<A: Action> ActionStack<A> {
    pub fn buffer_action(&mut self, action: A) {
        self.buffered_actions.push(action);
    }

    pub fn has_undo(&self) -> bool {
        !self.undo_actions.is_empty() && self.undo_enabled
    }

    pub fn has_redo(&self) -> bool {
        !self.redo_actions.is_empty() && self.redo_enabled
    }

    pub fn undo_action(&mut self, world: &mut World, entity_opt: Option<&Entity>) {
        if !self.undo_enabled {
            panic!("Undo is disabled!");
        }
        let Some(action) = self.undo_actions.pop() else {
            panic!("No executed actions to undo!");
        };

        let mut reversed_actions = self.execute_action(world, entity_opt, action);

        self.redo_actions.append(&mut reversed_actions);

        self.enable_top(world);
    }

    pub fn redo_action(&mut self, world: &mut World, entity_opt: Option<&Entity>) {
        if !self.redo_enabled {
            panic!("Redo is disabled!");
        }
        let Some(action) = self.redo_actions.pop() else {
            panic!("No undone actions to redo!");
        };

        let mut reversed_actions = self.execute_action(world, entity_opt, action);

        self.undo_actions.append(&mut reversed_actions);

        self.enable_top(world);
    }

    pub fn execute_actions(&mut self, world: &mut World, entity_opt: Option<&Entity>) {
        if self.buffered_check {
            self.enable_top(world);
            self.buffered_check = false;
        }
        if self.buffered_actions.is_empty() {
            return;
        }
        let drained_actions: Vec<A> = self.buffered_actions.drain(..).collect();
        for action in drained_actions {
            let mut reversed_actions = self.execute_action(world, entity_opt, action);
            self.undo_actions.append(&mut reversed_actions);
        }
        self.redo_actions.clear();

        self.enable_top(world);
    }

    fn execute_action(
        &mut self,
        world: &mut World,
        entity_opt: Option<&Entity>,
        action: A,
    ) -> Vec<A> {
        action.execute(world, entity_opt, self)
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
}

impl ActionStack<FileAction> {
    pub(crate) fn migrate_file_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        for action_list in [&mut self.undo_actions, &mut self.redo_actions] {
            for action in action_list.iter_mut() {
                action.migrate_file_entities(old_entity, new_entity);
            }
        }
    }
}

impl ActionStack<ShapeAction> {
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
