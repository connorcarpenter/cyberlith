use bevy_ecs::{entity::Entity, prelude::World, world::Mut};
use bevy_log::info;

use crate::app::resources::{
    action::model::ModelAction, canvas::Canvas, input::InputManager, model_manager::ModelManager,
};

pub fn execute(
    world: &mut World,
    input_manager: &mut InputManager,
    current_file_entity: &Entity,
    action: ModelAction,
) -> Vec<ModelAction> {
    let ModelAction::CreateModelTransform(edge_2d_entity, dependency_file_ext, dependency_file_entity) = action else {
        panic!("Expected CreateModelTransform");
    };

    info!("CreateModelTransform(edge_2d_entity: {:?}, dependency_file_ext: {:?}, dependency_file_entity: {:?})", edge_2d_entity, dependency_file_ext, dependency_file_entity);

    world.resource_scope(|world, mut model_manager: Mut<ModelManager>| {
        model_manager.create_networked_model_transform(
            world,
            input_manager,
            &edge_2d_entity,
            current_file_entity,
            &dependency_file_ext,
            &dependency_file_entity,
        );
    });

    let mut canvas = world.get_resource_mut::<Canvas>().unwrap();
    input_manager.deselect_shape(&mut canvas);
    input_manager.queue_resync_selection_ui();

    // TODO: migrate undo/redo entities

    return vec![ModelAction::DeleteModelTransform(edge_2d_entity)];
}
