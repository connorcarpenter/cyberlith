use bevy_ecs::{
    prelude::{Entity, World},
    query::With,
    system::{Commands, Query, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::{
    components::{ModelTransformControl, Vertex2d},
    resources::{
        action::model::ModelAction, canvas::Canvas, input::InputManager, shape_data::CanvasShape,
    },
};

pub(crate) fn execute(
    world: &mut World,
    input_manager: &mut InputManager,
    action: ModelAction,
) -> Vec<ModelAction> {
    let ModelAction::SelectShape(shape_2d_entity_opt) = action else {
        panic!("Expected SelectShape");
    };

    info!("SelectShape({:?})", shape_2d_entity_opt);

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
        Query<&ModelTransformControl, With<Vertex2d>>,
    )> = SystemState::new(world);
    let (mut commands, mut client, mut canvas, mtc_2d_q) = system_state.get_mut(world);

    let mut deselected_entity: Option<(Entity, CanvasShape)> = None;
    let mut entity_to_request = None;
    let mut entity_to_release = None;
    if let Some((shape_2d_entity, shape)) = input_manager.selected_shape_2d() {
        input_manager.deselect_shape(&mut canvas);
        deselected_entity = Some((shape_2d_entity, shape));

        match shape {
            CanvasShape::Vertex => {
                // deselected model transform control vertex?
                let mtc_entity = if let Ok(mtc) = mtc_2d_q.get(shape_2d_entity) {
                    mtc.model_transform_entity
                } else {
                    panic!("Expected MTC");
                };

                entity_to_release = Some(mtc_entity);
            }
            CanvasShape::Edge => {
                // deselected skel bone edge?
                todo!("check if it's a model transform control edge (rotation)");
            }
            _ => {}
        }
    }

    if let Some((shape_2d_entity, shape)) = shape_2d_entity_opt {
        input_manager.select_shape(&mut canvas, &shape_2d_entity, shape);

        match shape {
            CanvasShape::Vertex => {
                // selected model transform control vertex?
                let mtc_entity = if let Ok(mtc) = mtc_2d_q.get(shape_2d_entity) {
                    mtc.model_transform_entity
                } else {
                    panic!("Expected MTC");
                };

                entity_to_request = Some(mtc_entity);
            }
            CanvasShape::Edge => {
                // selected skel bone edge?
                // todo: check if it's a model transform control edge (rotation)
            }
            _ => {}
        }
    }

    // request/release auth
    if entity_to_request != entity_to_release {
        if let Some(entity) = entity_to_release {
            commands.entity(entity).release_authority(&mut client);
        }
        if let Some(entity) = entity_to_request {
            commands.entity(entity).request_authority(&mut client);
        }
    }

    system_state.apply(world);

    return vec![ModelAction::SelectShape(deselected_entity)];
}
