use bevy_ecs::{prelude::World, world::Mut};
use bevy_ecs::system::{Commands, ResMut, SystemState};

use bevy_log::info;
use naia_bevy_client::Client;

use crate::app::resources::{action::model::ModelAction, model_manager::ModelManager};

pub fn execute(
    world: &mut World,
    action: ModelAction,
) -> Vec<ModelAction> {
    let ModelAction::DeleteModelTransform(edge_2d_entity) = action else {
        panic!("Expected DeleteModelTransform");
    };

    info!(
        "DeleteModelTransform({:?})",
        edge_2d_entity,
    );

    let mut system_state: SystemState<(Commands, ResMut<ModelManager>)> = SystemState::new(world);
    let (mut commands, mut model_manager) = system_state.get_mut(world);

    let model_transform_entity = model_manager.model_transform_from_edge_2d(&edge_2d_entity).unwrap();

    commands.entity(model_transform_entity).despawn();

    model_manager.on_despawn_model_transform(&mut commands, &model_transform_entity);

    system_state.apply(world);

    // TODO: migrate undo/redo entities

    // TODO: store previous transform state here

    return vec![ModelAction::CreateModelTransform(edge_2d_entity)];
}
