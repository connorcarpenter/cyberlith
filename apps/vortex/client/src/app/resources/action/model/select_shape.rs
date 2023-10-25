use bevy_ecs::{
    prelude::{Entity, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use crate::app::resources::{
    action::model::ModelAction, canvas::Canvas,
    input::InputManager, shape_data::CanvasShape,
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

    let mut system_state: SystemState<ResMut<Canvas>> = SystemState::new(world);
    let mut canvas = system_state.get_mut(world);

    let mut deselected_entity: Option<(Entity, CanvasShape)> = None;
    if let Some((shape_2d_entity, shape_2d_type)) = input_manager.selected_shape_2d() {
        input_manager.deselect_shape(&mut canvas);
        deselected_entity = Some((shape_2d_entity, shape_2d_type));
    }

    // TODO: release auth?

    if let Some((shape_2d_entity, shape)) = shape_2d_entity_opt {
        input_manager.select_shape(&mut canvas, &shape_2d_entity, shape);
    }

    // TODO: request auth?

    system_state.apply(world);

    return vec![ModelAction::SelectShape(deselected_entity)];
}
