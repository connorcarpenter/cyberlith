use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::{ResMut, SystemState},
    world::Mut
};
use bevy_log::{info};

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{shape_manager::{CanvasShape, ShapeManager}, action::Action, action_stack::ActionStack};

pub(crate) fn execute(world: &mut World, shape_2d_entity_opt: Option<(Entity, CanvasShape)>) -> Vec<Action> {
    info!("SelectShape({:?})", shape_2d_entity_opt);

    let mut system_state: SystemState<(Commands, Client, ResMut<ShapeManager>)> =
        SystemState::new(world);
    let (mut commands, mut client, mut shape_manager) = system_state.get_mut(world);

    // Deselect all selected shapes, select the new selected shapes
    let (deselected_entity, entity_to_release) =
        ActionStack::deselect_all_selected_shapes(&mut shape_manager);
    let entity_to_request =
        ActionStack::select_shape(&mut shape_manager, shape_2d_entity_opt);

    if entity_to_request != entity_to_release {
        if let Some(entity) = entity_to_release {
            let mut entity_mut = commands.entity(entity);
            if entity_mut.authority(&client).is_some() {
                entity_mut.release_authority(&mut client);
            }
        }
        if let Some(entity) = entity_to_request {
            let mut entity_mut = commands.entity(entity);
            if entity_mut.authority(&client).is_some() {
                entity_mut.request_authority(&mut client);
            }
        }
    }

    system_state.apply(world);

    // create networked 3d face if necessary
    if let Some((face_2d_entity, CanvasShape::Face)) = shape_2d_entity_opt {
        if entity_to_request.is_none() {
            world.resource_scope(|world, mut shape_manager: Mut<ShapeManager>| {
                shape_manager.create_networked_face_outer(world, face_2d_entity);
            });
            return vec![
                Action::SelectShape(deselected_entity),
                Action::DeleteFace(face_2d_entity),
            ];
        }
    }

    return vec![Action::SelectShape(deselected_entity)];
}