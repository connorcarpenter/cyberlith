use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::{ResMut, SystemState},
};
use bevy_log::{info};

use render_api::{base::CpuMesh, Assets};

use crate::app::{
    resources::{
        action::Action,
        shape_manager::{CanvasShape, ShapeManager},
    },
};

pub(crate) fn execute(world: &mut World, face_2d_entity: Entity) -> Vec<Action> {
    info!("DeleteFace(face_2d_entity: `{:?}`)", face_2d_entity);
    let mut system_state: SystemState<(
        Commands,
        ResMut<ShapeManager>,
        ResMut<Assets<CpuMesh>>,
    )> = SystemState::new(world);
    let (mut commands, mut shape_manager, mut meshes) =
        system_state.get_mut(world);

    let Some(face_3d_entity) = shape_manager.face_entity_2d_to_3d(&face_2d_entity) else {
        panic!("failed to get face 3d entity for face 2d entity `{:?}`!", face_2d_entity)
    };

    // delete 3d face
    commands.entity(face_3d_entity).despawn();

    // cleanup mappings
    shape_manager.cleanup_deleted_face_3d(&mut commands, &mut meshes, &face_3d_entity);

    system_state.apply(world);

    return vec![Action::SelectShape(
        Some((face_2d_entity, CanvasShape::Face))
    )];
}