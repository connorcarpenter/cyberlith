use bevy_ecs::system::{Query, Res, SystemState};
use bevy_ecs::{entity::Entity, prelude::World, world::Mut};
use bevy_log::info;
use vortex_proto::components::ShapeName;

use crate::app::resources::{
    action::model::ModelAction, canvas::Canvas, edge_manager::EdgeManager, input::InputManager,
    model_manager::ModelManager,
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

    let mut system_state: SystemState<(Res<EdgeManager>, Query<&ShapeName>)> =
        SystemState::new(world);
    let (edge_manager, shape_name_q) = system_state.get_mut(world);

    let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();
    let (_, vertex_3d_entity) = edge_manager.edge_get_endpoints(&edge_3d_entity);
    let shape_name = shape_name_q.get(vertex_3d_entity).unwrap();
    let vertex_name = (*shape_name.value).clone();

    world.resource_scope(|world, mut model_manager: Mut<ModelManager>| {
        model_manager.create_networked_model_transform(
            world,
            input_manager,
            current_file_entity,
            &dependency_file_ext,
            &dependency_file_entity,
            vertex_name,
        );
    });

    let mut canvas = world.get_resource_mut::<Canvas>().unwrap();
    input_manager.deselect_shape(&mut canvas);
    input_manager.queue_resync_selection_ui();

    // TODO: migrate undo/redo entities

    return vec![ModelAction::DeleteModelTransform(edge_2d_entity)];
}
