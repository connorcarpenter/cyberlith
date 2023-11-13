use bevy_ecs::{
    event::EventWriter,
    prelude::{Commands, Entity, World},
    system::{Query, Res, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use vortex_proto::components::{FileExtension, IconFace};

use crate::app::{
    events::ShapeColorResyncEvent,
    resources::{
        action::icon::IconAction, file_manager::FileManager, icon_manager::IconManager,
        palette_manager::PaletteManager, shape_data::CanvasShape,
    },
};

pub(crate) fn execute(
    world: &mut World,
    icon_manager: &mut IconManager,
    current_file_entity: Entity,
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

    if let Some((local_face_entity, CanvasShape::Face)) = shape_entity_opt {
        let mut system_state: SystemState<(Res<FileManager>, Res<PaletteManager>)> =
            SystemState::new(world);
        let (file_manager, palette_manager) = system_state.get_mut(world);

        // create new face, assign color
        let palette_color_index = icon_manager.selected_color_index();
        let Some(palette_file_entity) = file_manager.file_get_dependency(
            &current_file_entity,
            FileExtension::Palette,
        ) else {
            panic!("Expected palette file dependency");
        };
        let next_palette_color_entity = palette_manager
            .get_color_entity(&palette_file_entity, palette_color_index)
            .unwrap();

        if let Some(net_face_entity) = icon_manager.face_entity_local_to_net(&local_face_entity) {
            let mut system_state: SystemState<(
                Client,
                Query<&mut IconFace>,
                EventWriter<ShapeColorResyncEvent>,
            )> = SystemState::new(world);
            let (client, mut face_q, mut shape_color_resync_event_writer) =
                system_state.get_mut(world);

            // edit face color

            let Ok(mut face_component) = face_q.get_mut(net_face_entity) else {
                panic!("Failed to get FaceColor for face entity {:?}!", net_face_entity);
            };

            let prev_palette_entity = face_component.palette_color_entity.get(&client).unwrap();

            face_component
                .palette_color_entity
                .set(&client, &next_palette_color_entity);

            shape_color_resync_event_writer.send(ShapeColorResyncEvent);

            return vec![
                IconAction::SelectShape(deselected_entity),
                IconAction::EditColor(local_face_entity, Some(prev_palette_entity)),
            ];
        } else {
            icon_manager.create_networked_face_from_world(
                world,
                local_face_entity,
                next_palette_color_entity,
            );

            return vec![
                IconAction::SelectShape(deselected_entity),
                IconAction::DeleteFace(local_face_entity),
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
