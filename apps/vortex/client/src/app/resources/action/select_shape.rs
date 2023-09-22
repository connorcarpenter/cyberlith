use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::{Res, ResMut, SystemState},
    world::Mut,
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{
    action::ShapeAction, canvas::Canvas, edge_manager::EdgeManager, face_manager::FaceManager,
    input_manager::InputManager, shape_data::CanvasShape, shape_manager::ShapeManager,
    vertex_manager::VertexManager,
};

pub(crate) fn execute(
    world: &mut World,
    input_manager: &mut InputManager,
    action: ShapeAction,
) -> Vec<ShapeAction> {
    let ShapeAction::SelectShape(shape_2d_entity_opt) = action else {
        panic!("Expected SelectShape");
    };

    info!("SelectShape({:?})", shape_2d_entity_opt);

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
        Res<VertexManager>,
        Res<EdgeManager>,
        Res<FaceManager>,
    )> = SystemState::new(world);
    let (mut commands, mut client, mut canvas, vertex_manager, edge_manager, face_manager) =
        system_state.get_mut(world);

    // Deselect all selected shapes, select the new selected shapes
    let (deselected_entity, entity_to_release) = deselect_selected_shape(
        &mut canvas,
        input_manager,
        &vertex_manager,
        &edge_manager,
        &face_manager,
    );
    let entity_to_request = select_shape(
        &mut canvas,
        input_manager,
        &vertex_manager,
        &edge_manager,
        &face_manager,
        shape_2d_entity_opt,
    );
    entity_request_release(
        &mut commands,
        &mut client,
        entity_to_request,
        entity_to_release,
    );

    system_state.apply(world);

    // create networked 3d face if necessary
    if let Some((face_2d_entity, CanvasShape::Face)) = shape_2d_entity_opt {
        if entity_to_request.is_none() {
            world.resource_scope(|world, mut face_manager: Mut<FaceManager>| {
                face_manager.create_networked_face_from_world(world, face_2d_entity);
            });
            return vec![
                ShapeAction::SelectShape(deselected_entity),
                ShapeAction::DeleteFace(face_2d_entity),
            ];
        }
    }

    return vec![ShapeAction::SelectShape(deselected_entity)];
}

pub fn entity_request_release(
    commands: &mut Commands,
    mut client: &mut Client,
    entity_to_request: Option<Entity>,
    entity_to_release: Option<Entity>,
) {
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
}

// returns entity to request auth for
pub fn select_shape(
    canvas: &mut Canvas,
    input_manager: &mut InputManager,
    vertex_manager: &VertexManager,
    edge_manager: &EdgeManager,
    face_manager: &FaceManager,
    shape_2d_entity_opt: Option<(Entity, CanvasShape)>,
) -> Option<Entity> {
    if let Some((shape_2d_entity, shape)) = shape_2d_entity_opt {
        input_manager.select_shape(canvas, &shape_2d_entity, shape);
        match shape {
            CanvasShape::Vertex => {
                let vertex_3d_entity = vertex_manager
                    .vertex_entity_2d_to_3d(&shape_2d_entity)
                    .unwrap();
                return Some(vertex_3d_entity);
            }
            CanvasShape::Edge => {
                let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&shape_2d_entity).unwrap();
                return Some(edge_3d_entity);
            }
            CanvasShape::Face => {
                return face_manager.face_entity_2d_to_3d(&shape_2d_entity);
            }
            _ => return None,
        }
    }
    return None;
}

pub fn deselect_selected_shape(
    canvas: &mut Canvas,
    input_manager: &mut InputManager,
    vertex_manager: &VertexManager,
    edge_manager: &EdgeManager,
    face_manager: &FaceManager,
) -> (Option<(Entity, CanvasShape)>, Option<Entity>) {
    let mut entity_to_deselect = None;
    let mut entity_to_release = None;
    if let Some((shape_2d_entity, shape_2d_type)) = input_manager.selected_shape_2d() {
        input_manager.deselect_shape(canvas);
        entity_to_deselect = Some((shape_2d_entity, shape_2d_type));
        entity_to_release = ShapeManager::shape_entity_2d_to_3d(
            vertex_manager,
            edge_manager,
            face_manager,
            &shape_2d_entity,
            shape_2d_type,
        );
    }
    (entity_to_deselect, entity_to_release)
}
