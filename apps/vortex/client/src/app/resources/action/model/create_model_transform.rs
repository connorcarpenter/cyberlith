use bevy_ecs::{prelude::World, world::Mut};

use bevy_log::info;

use crate::app::resources::{action::model::ModelAction, model_manager::ModelManager};

pub fn execute(
    world: &mut World,
    action: ModelAction,
) -> Vec<ModelAction> {
    let ModelAction::CreateModelTransform(edge_2d_entity) = action else {
        panic!("Expected CreateModelTransform");
    };

    info!(
        "CreateModelTransform({:?})",
        edge_2d_entity,
    );

    world.resource_scope(|world, mut model_manager: Mut<ModelManager>| {
        model_manager.create_networked_model_transform(world, edge_2d_entity);
    });

    // TODO: migrate undo/redo entities

    return vec![ModelAction::DeleteModelTransform(edge_2d_entity)];
}
