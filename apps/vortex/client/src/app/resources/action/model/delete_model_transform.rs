use bevy_ecs::{
    prelude::World,
    system::{Commands, Query, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::components::{FileExtension, ModelTransform, ModelTransformEntityType};

use crate::app::resources::{action::model::ModelAction, model_manager::ModelManager};

pub fn execute(world: &mut World, action: ModelAction) -> Vec<ModelAction> {
    let ModelAction::DeleteModelTransform(edge_2d_entity) = action else {
        panic!("Expected DeleteModelTransform");
    };

    info!("DeleteModelTransform({:?})", edge_2d_entity,);

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<ModelManager>,
        Query<&ModelTransform>,
    )> = SystemState::new(world);
    let (mut commands, client, mut model_manager, model_transform_q) = system_state.get_mut(world);

    let model_transform_entity = model_manager
        .model_transform_from_edge_2d(&edge_2d_entity)
        .unwrap();
    let model_transform = model_transform_q.get(model_transform_entity).unwrap();
    let dependency_file_entity = model_transform.skin_or_scene_entity.get(&client).unwrap();
    let dependency_file_ext = match *model_transform.entity_type {
        ModelTransformEntityType::Skin => FileExtension::Skin,
        ModelTransformEntityType::Scene => FileExtension::Scene,
        _ => panic!("Expected skin or scene"),
    };

    commands.entity(model_transform_entity).despawn();

    model_manager.on_despawn_model_transform(&mut commands, &model_transform_entity);

    system_state.apply(world);

    // TODO: migrate undo/redo entities

    // TODO: store previous transform state here

    return vec![ModelAction::CreateModelTransform(
        edge_2d_entity,
        dependency_file_ext,
        dependency_file_entity,
    )];
}
