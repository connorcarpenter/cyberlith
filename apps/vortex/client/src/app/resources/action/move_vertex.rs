use bevy_ecs::{
    prelude::{Entity, Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use math::Vec3;
use render_api::{base::CpuMesh, components::Transform, Assets, Handle};

use vortex_proto::components::{Face3d, Vertex3d};

use crate::app::resources::{
    action::ShapeAction, camera_manager::CameraManager, shape_manager::ShapeManager,
};

pub(crate) fn execute(
    world: &mut World,
    vertex_2d_entity: Entity,
    old_position: Vec3,
    new_position: Vec3,
) -> Vec<ShapeAction> {
    info!("MoveVertex");
    let mut system_state: SystemState<(
        Client,
        ResMut<Assets<CpuMesh>>,
        ResMut<ShapeManager>,
        ResMut<CameraManager>,
        //Query<&mut Vertex3d>,
        Query<&Handle<CpuMesh>>,
        Query<&Face3d>,
        Query<&mut Transform>,
        Query<&mut Vertex3d>,
    )> = SystemState::new(world);
    let (
        client,
        mut meshes,
        shape_manager,
        mut camera_manager,
        //mut vertex_3d_q,
        mesh_handle_q,
        face_3d_q,
        mut transform_q,
        mut vertex_3d_q,
    ) = system_state.get_mut(world);

    let vertex_3d_entity = vertex_manager
        .vertex_entity_2d_to_3d(&vertex_2d_entity)
        .unwrap();

    // MoveVertex action happens after the vertex has already been moved, so we wouldn't need to do anything here ..
    // BUT we do need to update the vertex_3d's position here in order to apply when undo/redo is executed
    let Ok(mut vertex_3d) = vertex_3d_q.get_mut(vertex_3d_entity) else {
        panic!("Failed to get Vertex3d for vertex entity {:?}!", vertex_3d_entity);
    };
    vertex_3d.set_vec3(&new_position);

    vertex_manager.on_vertex_3d_moved(
        &client,
        &mut meshes,
        &mesh_handle_q,
        &face_3d_q,
        &mut transform_q,
        &vertex_3d_entity,
    );

    camera_manager.recalculate_3d_view();

    system_state.apply(world);

    return vec![ShapeAction::MoveVertex(
        vertex_2d_entity,
        new_position,
        old_position,
    )];
}
