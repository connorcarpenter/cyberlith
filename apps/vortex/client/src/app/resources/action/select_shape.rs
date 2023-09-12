use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::{ResMut, SystemState},
    world::Mut,
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{
    shape_data::CanvasShape,
    action::ShapeAction,
    shape_manager::ShapeManager,
};

pub(crate) fn execute(
    world: &mut World,
    shape_2d_entity_opt: Option<(Entity, CanvasShape)>,
) -> Vec<ShapeAction> {
    info!("SelectShape({:?})", shape_2d_entity_opt);

    let mut system_state: SystemState<(Commands, Client, ResMut<ShapeManager>)> =
        SystemState::new(world);
    let (mut commands, mut client, mut shape_manager) = system_state.get_mut(world);

    // Deselect all selected shapes, select the new selected shapes
    let (deselected_entity, entity_to_release) = deselect_all_selected_shapes(&mut shape_manager);
    let entity_to_request = select_shape(&mut shape_manager, shape_2d_entity_opt);

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
                shape_manager.create_networked_face_from_world(world, face_2d_entity);
            });
            return vec![
                ShapeAction::SelectShape(deselected_entity),
                ShapeAction::DeleteFace(face_2d_entity),
            ];
        }
    }

    return vec![ShapeAction::SelectShape(deselected_entity)];
}

// returns entity to request auth for
pub fn select_shape(
    shape_manager: &mut ShapeManager,
    shape_2d_entity_opt: Option<(Entity, CanvasShape)>,
) -> Option<Entity> {
    if let Some((shape_2d_entity, shape)) = shape_2d_entity_opt {
        shape_manager.select_shape(&shape_2d_entity, shape);
        match shape {
            CanvasShape::Vertex => {
                let vertex_3d_entity = shape_manager
                    .vertex_entity_2d_to_3d(&shape_2d_entity)
                    .unwrap();
                return Some(vertex_3d_entity);
            }
            CanvasShape::Edge => {
                let edge_3d_entity = shape_manager
                    .edge_entity_2d_to_3d(&shape_2d_entity)
                    .unwrap();
                return Some(edge_3d_entity);
            }
            CanvasShape::Face => {
                return shape_manager.face_entity_2d_to_3d(&shape_2d_entity);
            }
            _ => return None,
        }
    }
    return None;
}

pub fn deselect_all_selected_shapes(
    shape_manager: &mut ShapeManager,
) -> (Option<(Entity, CanvasShape)>, Option<Entity>) {
    let mut entity_to_deselect = None;
    let mut entity_to_release = None;
    if let Some((shape_2d_entity, shape_2d_type)) = shape_manager.selected_shape_2d() {
        shape_manager.deselect_shape();
        entity_to_deselect = Some((shape_2d_entity, shape_2d_type));
        match shape_2d_type {
            CanvasShape::Vertex => {
                let vertex_3d_entity = shape_manager
                    .vertex_entity_2d_to_3d(&shape_2d_entity)
                    .unwrap();
                entity_to_release = Some(vertex_3d_entity);
            }
            CanvasShape::Edge => {
                let edge_3d_entity = shape_manager
                    .edge_entity_2d_to_3d(&shape_2d_entity)
                    .unwrap();
                entity_to_release = Some(edge_3d_entity);
            }
            CanvasShape::Face => {
                if let Some(face_3d_entity) = shape_manager.face_entity_2d_to_3d(&shape_2d_entity) {
                    entity_to_release = Some(face_3d_entity);
                }
            }
            _ => {}
        }
    }
    (entity_to_deselect, entity_to_release)
}
