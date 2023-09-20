use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::{Query, Res, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use vortex_proto::components::ShapeName;

use crate::app::resources::{canvas::Canvas, input_manager::InputManager,
    shape_data::CanvasShape, vertex_manager::VertexManager, action::AnimAction,
                            animation_manager::AnimationManager,
};


pub fn execute(
    world: &mut World,
    vertex_2d_entity_opt: Option<Entity>,
) -> Vec<AnimAction> {
    info!("SelectVertexAnim({:?})", vertex_2d_entity_opt);

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
        ResMut<InputManager>,
        Res<VertexManager>,
        Res<AnimationManager>,
        Query<&ShapeName>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut canvas,
        mut input_manager,
        vertex_manager,
        animation_manager,
        name_q,
    ) = system_state.get_mut(world);

    // Deselect all selected shapes, select the new selected shapes
    let (deselected_entity, entity_to_release) = deselect_selected_vertex(
        &mut canvas,
        &mut input_manager,
        &vertex_manager,
        &animation_manager,
        &name_q,
    );
    let entity_to_request = select_vertex(
        &mut canvas,
        &mut input_manager,
        &vertex_manager,
        &animation_manager,
        vertex_2d_entity_opt,
        &name_q,
    );

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

    return vec![AnimAction::SelectVertex(deselected_entity)];
}

// returns entity to request auth for
fn select_vertex(
    canvas: &mut Canvas,
    input_manager: &mut InputManager,
    vertex_manager: &VertexManager,
    animation_manager: &AnimationManager,
    vertex_2d_entity_opt: Option<Entity>,
    name_q: &Query<&ShapeName>,
) -> Option<Entity> {
    let vertex_2d_entity = vertex_2d_entity_opt?;
    input_manager.select_shape(canvas, &vertex_2d_entity, CanvasShape::Vertex);
    let vertex_3d_entity = vertex_manager
        .vertex_entity_2d_to_3d(&vertex_2d_entity)
        .unwrap();
    if let Ok(name) = name_q.get(vertex_3d_entity) {
        let name = name.value.as_str();
        return animation_manager.get_current_rotation(name).map(|entity| *entity);
    }
    return None;
}

fn deselect_selected_vertex(
    canvas: &mut Canvas,
    input_manager: &mut InputManager,
    vertex_manager: &VertexManager,
    animation_manager: &AnimationManager,
    name_q: &Query<&ShapeName>,
) -> (Option<Entity>, Option<Entity>) {
    let mut entity_to_deselect = None;
    let mut entity_to_release = None;
    if let Some((vertex_2d_entity, shape_2d_type)) = input_manager.selected_shape_2d() {
        if shape_2d_type != CanvasShape::Vertex && shape_2d_type != CanvasShape::RootVertex {
            panic!("only vertex shapes should be selected")
        }
        input_manager.deselect_shape(canvas);
        entity_to_deselect = Some(vertex_2d_entity);

        let vertex_3d_entity = vertex_manager
            .vertex_entity_2d_to_3d(&vertex_2d_entity)
            .unwrap();
        if let Ok(name) = name_q.get(vertex_3d_entity) {
            let name = name.value.as_str();
            if let Some(rotation_entity) = animation_manager.get_current_rotation(name) {
                entity_to_release = Some(*rotation_entity);
            }
        }
    }
    (entity_to_deselect, entity_to_release)
}
