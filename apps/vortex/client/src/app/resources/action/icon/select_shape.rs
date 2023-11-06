use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::{ResMut, SystemState},
    world::Mut,
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{
    action::icon::IconAction, canvas::Canvas,
    input::InputManager, shape_data::CanvasShape,
    icon_manager::IconManager
};

pub(crate) fn execute(
    world: &mut World,
    input_manager: &mut InputManager,
    action: IconAction,
) -> Vec<IconAction> {
    let IconAction::SelectShape(shape_entity_opt) = action else {
        panic!("Expected SelectShape");
    };

    info!("SelectShape({:?})", shape_entity_opt);

    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut canvas,
    ) = system_state.get_mut(world);

    // Deselect all selected shapes, select the new selected shapes
    let deselected_entity = deselect_selected_shape(
        &mut canvas,
        input_manager,
    );
    let entity_to_request = select_shape(
        &mut canvas,
        input_manager,
        shape_entity_opt,
    );
    let entity_to_release = deselected_entity.map(|(entity, _)| {
        entity
    });
    entity_request_release(
        &mut commands,
        &mut client,
        entity_to_request,
        entity_to_release,
    );

    system_state.apply(world);

    // create net face if necessary
    if let Some((face_entity, CanvasShape::Face)) = shape_entity_opt {
        if entity_to_request.is_none() {
            world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                icon_manager.create_networked_face_from_world(world, face_entity);
            });
            return vec![
                IconAction::SelectShape(deselected_entity),
                IconAction::DeleteFace(face_entity),
            ];
        }
    }

    return vec![IconAction::SelectShape(deselected_entity)];
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
    shape_entity_opt: Option<(Entity, CanvasShape)>,
) -> Option<Entity> {
    if let Some((shape_entity, shape)) = shape_entity_opt {
        input_manager.select_shape(canvas, &shape_entity, shape);
        return Some(shape_entity);
    }
    return None;
}

pub fn deselect_selected_shape(
    canvas: &mut Canvas,
    input_manager: &mut InputManager,
) -> Option<(Entity, CanvasShape)> {
    let mut entity_to_deselect = None;
    if let Some((shape_entity, shape_type)) = input_manager.selected_shape_2d() {
        input_manager.deselect_shape(canvas);
        entity_to_deselect = Some((shape_entity, shape_type));
    }
    entity_to_deselect
}
