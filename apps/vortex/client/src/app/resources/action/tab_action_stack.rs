use bevy_ecs::{
    prelude::{Entity, World},
    world::Mut,
};

use vortex_proto::components::FileExtension;

use crate::app::resources::{
    action::{animation::AnimAction, palette::PaletteAction, shape::ShapeAction, skin::SkinAction, ActionStack},
    input_manager::InputManager,
    palette_manager::PaletteManager,
};

pub enum TabActionStack {
    Shape(ActionStack<ShapeAction>),
    Animation(ActionStack<AnimAction>),
    Palette(ActionStack<PaletteAction>),
    Skin(ActionStack<SkinAction>),
}

impl TabActionStack {
    pub fn new(file_ext: FileExtension) -> Self {
        match file_ext {
            FileExtension::Skel | FileExtension::Mesh => Self::Shape(ActionStack::default()),
            FileExtension::Anim => Self::Animation(ActionStack::default()),
            FileExtension::Palette => Self::Palette(ActionStack::default()),
            FileExtension::Skin => Self::Skin(ActionStack::default()),
            _ => {
                panic!(
                    "TabActionStack::new() called with unsupported file extension: {:?}",
                    file_ext
                );
            }
        }
    }

    pub fn execute_shape_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        tab_file_entity: Entity,
        action: ShapeAction,
    ) {
        match self {
            Self::Shape(action_stack) => {
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_action_execution(world, reversed_actions);
            }
            _ => {
                panic!("buffer_shape_action() called on TabActionStack::Shape");
            }
        }
    }

    pub fn execute_anim_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        tab_file_entity: Entity,
        action: AnimAction,
    ) {
        match self {
            Self::Animation(action_stack) => {
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_action_execution(world, reversed_actions);
            }
            _ => {
                panic!("buffer_anim_action() called on TabActionStack::Animation");
            }
        }
    }

    pub fn execute_palette_action(
        &mut self,
        world: &mut World,
        palette_manager: &mut PaletteManager,
        action: PaletteAction,
    ) {
        match self {
            Self::Palette(action_stack) => {
                let reversed_actions = action_stack.execute_action(world, palette_manager, action);
                action_stack.post_action_execution(world, reversed_actions);
            }
            _ => {
                panic!("buffer_anim_action() called on TabActionStack::Palette");
            }
        }
    }

    pub fn has_undo(&self) -> bool {
        match self {
            Self::Shape(action_stack) => action_stack.has_undo(),
            Self::Animation(action_stack) => action_stack.has_undo(),
            Self::Palette(action_stack) => action_stack.has_undo(),
            Self::Skin(action_stack) => action_stack.has_undo(),
        }
    }

    pub fn has_redo(&self) -> bool {
        match self {
            Self::Shape(action_stack) => action_stack.has_redo(),
            Self::Animation(action_stack) => action_stack.has_redo(),
            Self::Palette(action_stack) => action_stack.has_redo(),
            Self::Skin(action_stack) => action_stack.has_redo(),
        }
    }

    pub fn undo_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        tab_file_entity: Entity,
    ) {
        match self {
            Self::Shape(action_stack) => {
                let action = action_stack.pop_undo();
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_execute_undo(world, reversed_actions);
            }
            Self::Animation(action_stack) => {
                let action = action_stack.pop_undo();
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_execute_undo(world, reversed_actions);
            }
            Self::Palette(action_stack) => {
                let action = action_stack.pop_undo();
                world.resource_scope(|world, mut palette_manager: Mut<PaletteManager>| {
                    let reversed_actions =
                        action_stack.execute_action(world, &mut palette_manager, action);
                    action_stack.post_execute_undo(world, reversed_actions);
                });
            }
            Self::Skin(action_stack) => {
                let action = action_stack.pop_undo();
                let reversed_actions = action_stack.execute_action(world, action);
                action_stack.post_execute_undo(world, reversed_actions);
            }
        };
    }

    pub fn redo_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        tab_file_entity: Entity,
    ) {
        match self {
            Self::Shape(action_stack) => {
                let action = action_stack.pop_redo();
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_execute_redo(world, reversed_actions);
            }
            Self::Animation(action_stack) => {
                let action = action_stack.pop_redo();
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_execute_redo(world, reversed_actions);
            }
            Self::Palette(action_stack) => {
                let action = action_stack.pop_redo();
                world.resource_scope(|world, mut palette_manager: Mut<PaletteManager>| {
                    let reversed_actions =
                        action_stack.execute_action(world, &mut palette_manager, action);
                    action_stack.post_execute_redo(world, reversed_actions);
                });
            }
            Self::Skin(action_stack) => {
                let action = action_stack.pop_redo();
                let reversed_actions = action_stack.execute_action(world, action);
                action_stack.post_execute_redo(world, reversed_actions);
            }
        }
    }

    pub fn check_top(&mut self, world: &mut World) {
        match self {
            Self::Shape(action_stack) => {
                action_stack.check_top(world);
            }
            Self::Animation(action_stack) => {
                action_stack.check_top(world);
            }
            Self::Palette(action_stack) => {
                action_stack.check_top(world);
            }
            Self::Skin(action_stack) => {
                action_stack.check_top(world);
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
            Self::Palette(action_stack) => {
                action_stack.entity_update_auth_status(entity);
            }
            Self::Skin(action_stack) => {
                action_stack.entity_update_auth_status(entity);
            }
        }
    }
}
