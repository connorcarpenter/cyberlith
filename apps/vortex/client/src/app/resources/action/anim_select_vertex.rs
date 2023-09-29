use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::{Query, Res, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::components::ShapeName;

use crate::app::resources::{
    action::{select_shape::entity_request_release, AnimAction},
    animation_manager::AnimationManager,
    canvas::Canvas,
    edge_manager::EdgeManager,
    input_manager::InputManager,
    shape_data::CanvasShape,
    vertex_manager::VertexManager,
};

pub fn execute(
    world: &mut World,
    input_manager: &mut InputManager,
    tab_file_entity: Entity,
    action: AnimAction,
) -> Vec<AnimAction> {
    let AnimAction::SelectShape(shape_2d_entity_opt) = action else {
        panic!("Expected SelectShape");
    };

    info!("AnimSelectShape({:?})", shape_2d_entity_opt);

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
        Res<VertexManager>,
        Res<EdgeManager>,
        Res<AnimationManager>,
        Query<&ShapeName>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut canvas,
        vertex_manager,
        edge_manager,
        animation_manager,
        name_q,
    ) = system_state.get_mut(world);

    let file_entity = tab_file_entity;

    // Deselect all selected shapes, select the new selected shapes
    let (deselected_entity, entity_to_release) = deselect_selected_shape(
        &mut canvas,
        input_manager,
        &vertex_manager,
        &edge_manager,
        &animation_manager,
        &name_q,
        &file_entity,
    );
    let entity_to_request = select_shape(
        &mut canvas,
        input_manager,
        &vertex_manager,
        &edge_manager,
        &animation_manager,
        &file_entity,
        shape_2d_entity_opt,
        &name_q,
    );
    entity_request_release(
        &mut commands,
        &mut client,
        entity_to_request,
        entity_to_release,
    );

    system_state.apply(world);

    return vec![AnimAction::SelectShape(deselected_entity)];
}

// returns entity to request auth for
fn select_shape(
    canvas: &mut Canvas,
    input_manager: &mut InputManager,
    vertex_manager: &VertexManager,
    edge_manager: &EdgeManager,
    animation_manager: &AnimationManager,
    file_entity: &Entity,
    shape_2d_entity_opt: Option<(Entity, CanvasShape)>,
    name_q: &Query<&ShapeName>,
) -> Option<Entity> {
    let (shape_2d_entity, shape) = shape_2d_entity_opt?;
    input_manager.select_shape(canvas, &shape_2d_entity, shape);

    match shape {
        CanvasShape::Vertex => {
            let vertex_3d_entity = vertex_manager
                .vertex_entity_2d_to_3d(&shape_2d_entity)
                .unwrap();
            return get_rotation_entity(animation_manager, name_q, file_entity, vertex_3d_entity);
        }
        CanvasShape::Edge => {
            let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&shape_2d_entity).unwrap();
            let (_, vertex_3d_entity) = edge_manager.edge_get_endpoints(&edge_3d_entity);
            return get_rotation_entity(animation_manager, name_q, file_entity, vertex_3d_entity);
        }
        _ => {}
    }

    return None;
}

fn get_rotation_entity(
    animation_manager: &AnimationManager,
    name_q: &Query<&ShapeName>,
    file_entity: &Entity,
    vertex_3d_entity: Entity,
) -> Option<Entity> {
    let name = name_q.get(vertex_3d_entity).ok()?;
    let name = name.value.as_str();
    return animation_manager
        .get_current_rotation(file_entity,name)
        .map(|entity| *entity);
}

fn deselect_selected_shape(
    canvas: &mut Canvas,
    input_manager: &mut InputManager,
    vertex_manager: &VertexManager,
    edge_manager: &EdgeManager,
    animation_manager: &AnimationManager,
    name_q: &Query<&ShapeName>,
    file_entity: &Entity,
) -> (Option<(Entity, CanvasShape)>, Option<Entity>) {
    let mut entity_to_deselect = None;
    let mut entity_to_release = None;
    if let Some((shape_2d_entity, shape_2d_type)) = input_manager.selected_shape_2d() {
        input_manager.deselect_shape(canvas);
        entity_to_deselect = Some((shape_2d_entity, shape_2d_type));

        match shape_2d_type {
            CanvasShape::RootVertex | CanvasShape::Vertex => {
                let vertex_3d_entity = vertex_manager
                    .vertex_entity_2d_to_3d(&shape_2d_entity)
                    .unwrap();
                entity_to_release =
                    get_rotation_entity(animation_manager, name_q, file_entity, vertex_3d_entity);
            }
            CanvasShape::Edge => {
                let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&shape_2d_entity).unwrap();
                let (_, vertex_3d_entity) = edge_manager.edge_get_endpoints(&edge_3d_entity);
                entity_to_release =
                    get_rotation_entity(animation_manager, name_q, file_entity, vertex_3d_entity);
            }
            CanvasShape::Face => {
                panic!("Unexpected shape type");
            }
        }
    }
    (entity_to_deselect, entity_to_release)
}
