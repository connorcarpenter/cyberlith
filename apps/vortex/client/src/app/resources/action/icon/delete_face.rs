use bevy_ecs::{
    prelude::{Commands, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use render_api::{base::CpuMesh, Assets};

use crate::app::resources::{
    action::icon::IconAction, icon_manager::IconManager, shape_data::CanvasShape,
};

pub(crate) fn execute(
    world: &mut World,
    icon_manager: &mut IconManager,
    action: IconAction,
) -> Vec<IconAction> {
    let IconAction::DeleteFace(local_face_entity) = action else {
        panic!("Expected DeleteFace");
    };

    info!("DeleteFace(local_face_entity: `{:?}`)", local_face_entity);
    let mut system_state: SystemState<(Commands, ResMut<Assets<CpuMesh>>)> =
        SystemState::new(world);
    let (mut commands, mut meshes) = system_state.get_mut(world);

    let Some(net_face_entity) = icon_manager.face_entity_local_to_net(&local_face_entity) else {
        panic!("failed to get net face entity for local face entity `{:?}`!", local_face_entity)
    };

    // delete net face
    commands.entity(net_face_entity).despawn();

    // cleanup mappings
    icon_manager.cleanup_deleted_net_face(&mut commands, &mut meshes, &net_face_entity);

    system_state.apply(world);

    return vec![IconAction::SelectShape(Some((
        local_face_entity,
        CanvasShape::Face,
    )))];
}
