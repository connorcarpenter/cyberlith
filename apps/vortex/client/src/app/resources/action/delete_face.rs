use bevy_ecs::{
    prelude::{Commands, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use render_api::{base::CpuMesh, Assets};

use crate::app::resources::{
    action::ShapeAction, face_manager::FaceManager, shape_data::CanvasShape,
};

pub(crate) fn execute(world: &mut World, action: ShapeAction) -> Vec<ShapeAction> {
    let ShapeAction::DeleteFace(face_2d_entity) = action else {
        panic!("Expected DeleteFace");
    };

    //, face_2d_entity: Entity

    info!("DeleteFace(face_2d_entity: `{:?}`)", face_2d_entity);
    let mut system_state: SystemState<(Commands, ResMut<FaceManager>, ResMut<Assets<CpuMesh>>)> =
        SystemState::new(world);
    let (mut commands, mut face_manager, mut meshes) = system_state.get_mut(world);

    let Some(face_3d_entity) = face_manager.face_entity_2d_to_3d(&face_2d_entity) else {
        panic!("failed to get face 3d entity for face 2d entity `{:?}`!", face_2d_entity)
    };

    // delete 3d face
    commands.entity(face_3d_entity).despawn();

    // cleanup mappings
    face_manager.cleanup_deleted_face_3d(&mut commands, &mut meshes, &face_3d_entity);

    system_state.apply(world);

    return vec![ShapeAction::SelectShape(Some((
        face_2d_entity,
        CanvasShape::Face,
    )))];
}
