use bevy_ecs::{prelude::World, world::Mut};

use bevy_log::info;

use crate::app::resources::{
    action::model::ModelAction, input::InputManager, model_manager::ModelManager,
};

pub fn execute(
    world: &mut World,
    input_manager: &mut InputManager,
    action: ModelAction,
) -> Vec<ModelAction> {
    let ModelAction::CreateModelTransform(edge_2d_entity, dependency_file_ext, dependency_file_entity) = action else {
        panic!("Expected CreateModelTransform");
    };

    info!("CreateModelTransform({:?})", edge_2d_entity,);

    world.resource_scope(|world, mut model_manager: Mut<ModelManager>| {
        model_manager.create_networked_model_transform(
            world,
            input_manager,
            &edge_2d_entity,
            &dependency_file_ext,
            &dependency_file_entity,
        );
    });

    // TODO: migrate undo/redo entities

    return vec![ModelAction::DeleteModelTransform(edge_2d_entity)];
}
