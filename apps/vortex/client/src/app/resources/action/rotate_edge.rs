use bevy_ecs::{
    prelude::{Entity, World},
    system::{Query, ResMut, SystemState},
};
use bevy_log::info;

use vortex_proto::components::EdgeAngle;

use crate::app::resources::{
    action::ShapeAction, camera_manager::CameraManager, shape_manager::ShapeManager,
};

pub(crate) fn execute(
    world: &mut World,
    edge_2d_entity: Entity,
    old_angle: f32,
    new_angle: f32,
) -> Vec<ShapeAction> {
    info!(
        "RotateEdge(edge_2d_entity: `{:?}`, old_angle: `{:?}`, new_angle: `{:?}`)",
        edge_2d_entity, old_angle, new_angle
    );
    let mut system_state: SystemState<(
        ResMut<ShapeManager>,
        ResMut<CameraManager>,
        Query<&mut EdgeAngle>,
    )> = SystemState::new(world);
    let (shape_manager, mut camera_manager, mut edge_angle_q) = system_state.get_mut(world);

    let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();

    let Ok(mut edge_angle) = edge_angle_q.get_mut(edge_3d_entity) else {
        panic!("Failed to get EdgeAngle for edge entity {:?}!", edge_3d_entity);
    };
    edge_angle.set_radians(new_angle);

    camera_manager.recalculate_3d_view();

    system_state.apply(world);

    return vec![ShapeAction::RotateEdge(
        edge_2d_entity,
        new_angle,
        old_angle,
    )];
}
