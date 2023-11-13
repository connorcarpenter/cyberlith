use bevy_ecs::{
    event::EventWriter,
    prelude::World,
    system::{Query, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::components::IconFace;

use crate::app::{
    events::ShapeColorResyncEvent,
    resources::{action::icon::IconAction, icon_manager::IconManager},
};

pub(crate) fn execute(
    world: &mut World,
    icon_manager: &mut IconManager,
    action: IconAction,
) -> Vec<IconAction> {
    let IconAction::EditColor(local_face_entity, new_palette_color_opt) = action else {
        panic!("Expected EditColor");
    };

    info!(
        "EditColor(local_face_entity: `{:?}`, new_color_opt: `{:?}`)",
        local_face_entity, new_palette_color_opt
    );
    let mut system_state: SystemState<(
        Client,
        EventWriter<ShapeColorResyncEvent>,
        Query<&mut IconFace>,
    )> = SystemState::new(world);
    let (client, mut shape_color_resync_event_writer, mut face_q) = system_state.get_mut(world);

    let net_face_entity = icon_manager
        .face_entity_local_to_net(&local_face_entity)
        .unwrap();
    let Ok(mut face) = face_q.get_mut(net_face_entity) else {
        panic!("Failed to get IconFace for face entity {:?}!", net_face_entity);
    };
    let old_palette_color_entity_opt = face.palette_color_entity.get(&client);

    if let Some(new_palette_color_entity) = new_palette_color_opt {
        face.palette_color_entity
            .set(&client, &new_palette_color_entity);
    } else {
        face.palette_color_entity.set_to_none();
    }

    shape_color_resync_event_writer.send(ShapeColorResyncEvent);

    system_state.apply(world);

    return vec![IconAction::EditColor(
        local_face_entity,
        old_palette_color_entity_opt,
    )];
}
