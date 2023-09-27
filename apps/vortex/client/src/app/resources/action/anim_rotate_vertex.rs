use bevy_ecs::{
    prelude::{Commands, World},
    system::{Query, Res, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::components::{AnimRotation, ShapeName};

use crate::app::resources::{
    action::AnimAction, animation_manager::AnimationManager, canvas::Canvas,
    vertex_manager::VertexManager,
};

pub fn execute(world: &mut World, action: AnimAction) -> Vec<AnimAction> {
    let AnimAction::RotateVertex(vertex_2d_entity, old_angle_opt, new_angle_opt) = action else {
        panic!("Expected RotateVertex");
    };

    info!(
        "AnimRotateVertex({:?}, {:?}, {:?})",
        vertex_2d_entity, old_angle_opt, new_angle_opt
    );

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
        Res<VertexManager>,
        ResMut<AnimationManager>,
        Query<&ShapeName>,
        Query<&mut AnimRotation>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut canvas,
        vertex_manager,
        mut animation_manager,
        name_q,
        mut rotation_q,
    ) = system_state.get_mut(world);

    let vertex_3d_entity = vertex_manager
        .vertex_entity_2d_to_3d(&vertex_2d_entity)
        .unwrap();

    let Ok(name) = name_q.get(vertex_3d_entity) else {
        panic!("Failed to get ShapeName for vertex entity {:?}!", vertex_3d_entity);
    };
    let name = name.value.as_str();

    if old_angle_opt.is_some() {
        let rotation_entity = animation_manager.get_current_rotation(name).unwrap();
        let rotation_entity = *rotation_entity;
        if let Some(new_quat) = new_angle_opt {
            let mut rotation = rotation_q.get_mut(rotation_entity).unwrap();
            rotation.set_rotation(new_quat);
        } else {
            // despawn rotation
            commands.entity(rotation_entity).despawn();
            animation_manager.deregister_rotation(&rotation_entity);
        }
    } else {
        let new_quat = new_angle_opt.unwrap();

        if let Some(rotation_entity) = animation_manager.get_current_rotation(name) {
            // already has a rotation entity, so just update it
            let mut rotation = rotation_q.get_mut(*rotation_entity).unwrap();
            rotation.set_rotation(new_quat);
        } else {
            // create new rotation entity
            let frame_entity = animation_manager.current_frame().unwrap();
            animation_manager.create_networked_rotation(
                &mut commands,
                &mut client,
                frame_entity,
                name.to_string(),
                new_quat,
            );
        }
    }

    canvas.queue_resync_shapes();

    system_state.apply(world);

    return vec![AnimAction::RotateVertex(
        vertex_2d_entity,
        new_angle_opt,
        old_angle_opt,
    )];
}
