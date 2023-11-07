use bevy_ecs::{
    prelude::{Commands, Entity, World},
    system::SystemState,
    world::Mut,
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use crate::app::resources::{
    action::icon::IconAction, icon_manager::IconManager, shape_data::CanvasShape,
};

pub(crate) fn execute(
    world: &mut World,
    icon_manager: &mut IconManager,
    action: IconAction,
) -> Vec<IconAction> {
    let IconAction::SelectShape(shape_entity_opt) = action else {
        panic!("Expected SelectShape");
    };

    info!("SelectShape({:?})", shape_entity_opt);

    let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
    let (mut commands, mut client) = system_state.get_mut(world);

    // Deselect all selected shapes, select the new selected shapes
    let deselected_entity = deselect_selected_shape(icon_manager);
    let entity_to_request = select_shape(icon_manager, shape_entity_opt);
    let entity_to_release = deselected_entity.map(|(entity, _)| entity);
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
            icon_manager.create_networked_face_from_world(world, face_entity);
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
    icon_manager: &mut IconManager,
    shape_entity_opt: Option<(Entity, CanvasShape)>,
) -> Option<Entity> {
    if let Some((shape_entity, shape)) = shape_entity_opt {
        icon_manager.select_shape(&shape_entity, shape);

        match shape {
            CanvasShape::Vertex | CanvasShape::Edge => {
                return Some(shape_entity);
            }
            CanvasShape::Face => {
                return icon_manager.face_entity_local_to_net(&shape_entity);
            }
            _ => return None,
        }
    }
    return None;
}

pub fn deselect_selected_shape(icon_manager: &mut IconManager) -> Option<(Entity, CanvasShape)> {
    let mut entity_to_deselect = None;
    if let Some((shape_entity, shape_type)) = icon_manager.selected_shape() {
        icon_manager.deselect_shape();
        entity_to_deselect = Some((shape_entity, shape_type));
    }
    entity_to_deselect
}
