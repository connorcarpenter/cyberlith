use bevy_ecs::{
    prelude::{Entity, World},
    world::Mut,
};

use editor_proto::components::FileExtension;

use crate::app::resources::{
    action::{
        animation::AnimAction, icon::IconAction, model::ModelAction, palette::PaletteAction,
        shape::ShapeAction, skin::SkinAction, ActionStack,
    },
    icon_manager::IconManager,
    input::InputManager,
    palette_manager::PaletteManager,
};

pub enum TabActionStack {
    Shape(ActionStack<ShapeAction>),
    Animation(ActionStack<AnimAction>),
    Palette(ActionStack<PaletteAction>),
    Skin(ActionStack<SkinAction>),
    Model(ActionStack<ModelAction>),
    Icon(ActionStack<IconAction>),
}

impl TabActionStack {
    pub fn new(file_ext: FileExtension) -> Self {
        match file_ext {
            FileExtension::Skel | FileExtension::Mesh => Self::Shape(ActionStack::default()),
            FileExtension::Anim => Self::Animation(ActionStack::default()),
            FileExtension::Palette => Self::Palette(ActionStack::default()),
            FileExtension::Skin => Self::Skin(ActionStack::default()),
            FileExtension::Model | FileExtension::Scene => Self::Model(ActionStack::default()),
            FileExtension::Icon => Self::Icon(ActionStack::default()),
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
                panic!("execute_shape_action() called on non-TabActionStack::Shape");
            }
        }
    }

    pub fn execute_skin_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        tab_file_entity: Entity,
        action: SkinAction,
    ) {
        match self {
            Self::Skin(action_stack) => {
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_action_execution(world, reversed_actions);
            }
            _ => {
                panic!("execute_skin_action() called on non-TabActionStack::Skin");
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

    pub fn execute_model_action(
        &mut self,
        world: &mut World,
        input_manager: &mut InputManager,
        tab_file_entity: Entity,
        action: ModelAction,
    ) {
        match self {
            Self::Model(action_stack) => {
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_action_execution(world, reversed_actions);
            }
            _ => {
                panic!("execute_model_action() called on non-TabActionStack::Model");
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

    pub fn execute_icon_action(
        &mut self,
        world: &mut World,
        icon_manager: &mut IconManager,
        tab_file_entity: Entity,
        action: IconAction,
    ) {
        match self {
            Self::Icon(action_stack) => {
                let reversed_actions =
                    action_stack.execute_action(world, icon_manager, tab_file_entity, action);
                action_stack.post_action_execution(world, reversed_actions);
            }
            _ => {
                panic!("execute_icon_action() called on non-TabActionStack::Icon");
            }
        }
    }

    pub fn has_undo(&self) -> bool {
        match self {
            Self::Shape(action_stack) => action_stack.has_undo(),
            Self::Animation(action_stack) => action_stack.has_undo(),
            Self::Palette(action_stack) => action_stack.has_undo(),
            Self::Skin(action_stack) => action_stack.has_undo(),
            Self::Model(action_stack) => action_stack.has_undo(),
            Self::Icon(action_stack) => action_stack.has_undo(),
        }
    }

    pub fn has_redo(&self) -> bool {
        match self {
            Self::Shape(action_stack) => action_stack.has_redo(),
            Self::Animation(action_stack) => action_stack.has_redo(),
            Self::Palette(action_stack) => action_stack.has_redo(),
            Self::Skin(action_stack) => action_stack.has_redo(),
            Self::Model(action_stack) => action_stack.has_redo(),
            Self::Icon(action_stack) => action_stack.has_redo(),
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
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_execute_undo(world, reversed_actions);
            }
            Self::Model(action_stack) => {
                let action = action_stack.pop_undo();
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_execute_undo(world, reversed_actions);
            }
            Self::Icon(action_stack) => {
                let action = action_stack.pop_undo();
                world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                    let reversed_actions = action_stack.execute_action(
                        world,
                        &mut icon_manager,
                        tab_file_entity,
                        action,
                    );
                    action_stack.post_execute_undo(world, reversed_actions);
                });
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
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_execute_redo(world, reversed_actions);
            }
            Self::Model(action_stack) => {
                let action = action_stack.pop_redo();
                let reversed_actions =
                    action_stack.execute_action(world, input_manager, tab_file_entity, action);
                action_stack.post_execute_redo(world, reversed_actions);
            }
            Self::Icon(action_stack) => {
                let action = action_stack.pop_redo();
                world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                    let reversed_actions = action_stack.execute_action(
                        world,
                        &mut icon_manager,
                        tab_file_entity,
                        action,
                    );
                    action_stack.post_execute_redo(world, reversed_actions);
                });
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
            Self::Model(action_stack) => {
                action_stack.check_top(world);
            }
            Self::Icon(action_stack) => {
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
            Self::Model(action_stack) => {
                action_stack.entity_update_auth_status(entity);
            }
            Self::Icon(action_stack) => {
                action_stack.entity_update_auth_status(entity);
            }
        }
    }
}
