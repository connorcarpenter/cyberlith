use bevy_ecs::{
    entity::Entity,
    prelude::World,
    system::{Commands, Query, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::components::{FileExtension, NetTransformEntityType, ShapeName, SkinOrSceneEntity};

use crate::app::resources::{
    action::model::ModelAction, canvas::Canvas, edge_manager::EdgeManager, input::InputManager,
    model_manager::ModelManager, vertex_manager::VertexManager,
};

pub fn execute(
    world: &mut World,
    model_file_entity: &Entity,
    action: ModelAction,
) -> Vec<ModelAction> {
    let ModelAction::DeleteTransform(edge_2d_entity) = action else {
        panic!("Expected DeleteTransform");
    };

    info!("DeleteTransform({:?})", edge_2d_entity,);

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
        ResMut<InputManager>,
        ResMut<VertexManager>,
        ResMut<EdgeManager>,
        ResMut<ModelManager>,
        Query<&ShapeName>,
        Query<&SkinOrSceneEntity>,
    )> = SystemState::new(world);
    let (
        mut commands,
        client,
        mut canvas,
        mut input_manager,
        mut vertex_manager,
        mut edge_manager,
        mut model_manager,
        shape_name_q,
        skin_or_scene_q,
    ) = system_state.get_mut(world);

    let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();
    let (_, vertex_3d_entity) = edge_manager.edge_get_endpoints(&edge_3d_entity);
    let vertex_name = shape_name_q.get(vertex_3d_entity).unwrap();
    let vertex_name = (*vertex_name.value).clone();

    let net_transform_entity = model_manager
        .find_net_transform(model_file_entity, &vertex_name)
        .unwrap();
    let skin_or_scene = skin_or_scene_q.get(net_transform_entity).unwrap();
    let dependency_file_entity = skin_or_scene.value.get(&client).unwrap();
    let dependency_file_ext = match *skin_or_scene.value_type {
        NetTransformEntityType::Skin => FileExtension::Skin,
        NetTransformEntityType::Scene => FileExtension::Scene,
        _ => panic!("Expected skin or scene"),
    };

    commands.entity(net_transform_entity).despawn();

    model_manager.on_despawn_net_transform(
        &mut commands,
        &mut canvas,
        &mut input_manager,
        &mut vertex_manager,
        &mut edge_manager,
        &net_transform_entity,
    );

    system_state.apply(world);

    // TODO: migrate undo/redo entities

    // TODO: store previous transform state here

    return vec![ModelAction::CreateTransform(
        edge_2d_entity,
        dependency_file_ext,
        dependency_file_entity,
    )];
}
