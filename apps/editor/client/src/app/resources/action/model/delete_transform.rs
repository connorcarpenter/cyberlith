use bevy_ecs::{
    entity::Entity,
    prelude::World,
    system::{Commands, Query, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use editor_proto::components::{FileExtension, NetTransformEntityType, SkinOrSceneEntity};

use crate::app::{
    plugin::Main,
    resources::{
        action::model::ModelAction, canvas::Canvas, edge_manager::EdgeManager, input::InputManager,
        model_manager::ModelManager, vertex_manager::VertexManager,
    },
};

pub fn execute(
    world: &mut World,
    _model_file_entity: &Entity,
    action: ModelAction,
) -> Vec<ModelAction> {
    let ModelAction::DeleteTransform(net_transform_entity) = action else {
        panic!("Expected DeleteTransform");
    };

    info!("DeleteTransform({:?})", net_transform_entity);

    let mut system_state: SystemState<(
        Commands,
        Client<Main>,
        ResMut<Canvas>,
        ResMut<InputManager>,
        ResMut<VertexManager>,
        ResMut<EdgeManager>,
        ResMut<ModelManager>,
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
        skin_or_scene_q,
    ) = system_state.get_mut(world);

    let skin_or_scene = skin_or_scene_q.get(net_transform_entity).unwrap();
    let dependency_file_entity = skin_or_scene.value.get(&client).unwrap();
    let dependency_file_ext = match *skin_or_scene.value_type {
        NetTransformEntityType::Skin => FileExtension::Skin,
        NetTransformEntityType::Scene => FileExtension::Scene,
        _ => panic!("Expected skin or scene"),
    };

    let edge_2d_entity_opt = model_manager.get_edge_2d_entity(&net_transform_entity);

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
        edge_2d_entity_opt,
        dependency_file_ext,
        dependency_file_entity,
    )];
}
