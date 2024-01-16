use bevy_ecs::{
    prelude::{Entity, World},
    system::{Commands, Query, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::{
    components::NetTransformControl,
    resources::{
        action::model::ModelAction, canvas::Canvas, input::InputManager, shape_data::CanvasShape,
    },
    plugin::Main,
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
        Client<Main>,
        ResMut<Canvas>,
        Query<&NetTransformControl>,
    )> = SystemState::new(world);
    let (mut commands, mut client, mut canvas, ntc_2d_q) = system_state.get_mut(world);

    let mut deselected_entity: Option<(Entity, CanvasShape)> = None;
    let mut entity_to_request = None;
    let mut entity_to_release = None;
    if let Some((shape_2d_entity, shape)) = input_manager.selected_shape_2d() {
        input_manager.deselect_shape(&mut canvas);
        deselected_entity = Some((shape_2d_entity, shape));

        let ntc_entity_opt = if let Ok(ntc) = ntc_2d_q.get(shape_2d_entity) {
            Some(ntc.net_transform_entity)
        } else {
            None
        };

        match shape {
            CanvasShape::Vertex => {
                // deselected net transform control vertex?
                let ntc_entity = ntc_entity_opt.expect("Expected NTC");
                entity_to_release = Some(ntc_entity);
            }
            CanvasShape::Edge => {
                if let Some(ntc_entity) = ntc_entity_opt {
                    // deselected net transform control edge (rotation)
                    entity_to_release = Some(ntc_entity);
                } else {
                    // deselected skel bone edge
                }
            }
            _ => {}
        }
    }

    if let Some((shape_2d_entity, shape)) = shape_2d_entity_opt {
        input_manager.select_shape(&mut canvas, &shape_2d_entity, shape);

        let ntc_entity_opt = if let Ok(ntc) = ntc_2d_q.get(shape_2d_entity) {
            Some(ntc.net_transform_entity)
        } else {
            None
        };

        match shape {
            CanvasShape::Vertex => {
                // selected net transform control vertex?
                let ntc_entity = ntc_entity_opt.expect("Expected NTC");
                entity_to_request = Some(ntc_entity);
            }
            CanvasShape::Edge => {
                if let Some(ntc_entity) = ntc_entity_opt {
                    // selected net transform control edge (rotation)
                    entity_to_request = Some(ntc_entity);
                } else {
                    // selected skel bone edge
                }
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
