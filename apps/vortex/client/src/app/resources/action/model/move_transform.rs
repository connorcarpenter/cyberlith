use bevy_ecs::{
    prelude::{Query, World},
    system::{ResMut, SystemState},
};
use bevy_log::info;

use vortex_proto::components::ModelTransform;

use crate::app::{resources::{
    action::model::ModelAction, canvas::Canvas,
}, components::ModelTransformLocal};

pub(crate) fn execute(world: &mut World, action: ModelAction) -> Vec<ModelAction> {
    let ModelAction::MoveTransform(transform_entity, old_transform, new_transform, already_moved) = action else {
        panic!("Expected MoveTransform");
    };

    info!(
        "MoveTransform({:?}, _, _, {})",
        transform_entity, already_moved
    );
    let mut system_state: SystemState<(
        ResMut<Canvas>,
        Query<&mut ModelTransform>,
    )> = SystemState::new(world);
    let (
        mut canvas,
        mut model_transform_q,
    ) = system_state.get_mut(world);

    if !already_moved {
        // MoveTransform action happens after the transform has already been moved, so we wouldn't need to do anything here ..
        // BUT we do need to update the ModelTransform's state here in order to apply when undo/redo is executed
        let Ok(mut model_transform) = model_transform_q.get_mut(transform_entity) else {
            panic!("Failed to get ModelTransform for entity {:?}!", transform_entity);
        };
        ModelTransformLocal::set_transform(&mut model_transform, &new_transform);
    }

    canvas.queue_resync_shapes();

    system_state.apply(world);

    return vec![ModelAction::MoveTransform(
        transform_entity,
        new_transform,
        old_transform,
        false,
    )];
}
