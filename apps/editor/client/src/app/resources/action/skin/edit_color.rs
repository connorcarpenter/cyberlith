use bevy_ecs::{
    event::EventWriter,
    prelude::World,
    system::{Commands, Query, Res, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use editor_proto::components::FaceColor;

use crate::app::{
    events::ShapeColorResyncEvent,
    resources::{
        action::skin::SkinAction, canvas::Canvas, face_manager::FaceManager,
        skin_manager::SkinManager,
    },
    plugin::Main,
};

pub(crate) fn execute(world: &mut World, action: SkinAction) -> Vec<SkinAction> {
    let SkinAction::EditColor(face_2d_entity, new_palette_color_opt) = action else {
        panic!("Expected EditColor");
    };

    info!(
        "EditColor(face_2d_entity: `{:?}`, new_color_opt: `{:?}`)",
        face_2d_entity, new_palette_color_opt
    );
    let mut system_state: SystemState<(
        Commands,
        Client<Main>,
        ResMut<Canvas>,
        Res<FaceManager>,
        ResMut<SkinManager>,
        EventWriter<ShapeColorResyncEvent>,
        Query<&mut FaceColor>,
    )> = SystemState::new(world);
    let (
        mut commands,
        client,
        mut canvas,
        face_manager,
        mut skin_manager,
        mut shape_color_resync_event_writer,
        mut face_color_q,
    ) = system_state.get_mut(world);

    let face_3d_entity = face_manager.face_entity_2d_to_3d(&face_2d_entity).unwrap();
    let face_color_entity = *skin_manager.face_to_color_entity(&face_3d_entity).unwrap();
    let Ok(mut face_color) = face_color_q.get_mut(face_color_entity) else {
        panic!("Failed to get FaceColor for face color entity {:?}!", face_color_entity);
    };
    let old_palette_color_entity_opt = face_color.palette_color_entity.get(&client);

    if let Some(new_palette_color_entity) = new_palette_color_opt {
        face_color
            .palette_color_entity
            .set(&client, &new_palette_color_entity);
    } else {
        if let Some(_old_palette_color_entity) = old_palette_color_entity_opt {
            // despawn face color
            commands.entity(face_color_entity).despawn();
            skin_manager.deregister_face_color(&face_color_entity);
        }
    }

    canvas.queue_resync_shapes();
    shape_color_resync_event_writer.send(ShapeColorResyncEvent);

    system_state.apply(world);

    return vec![SkinAction::EditColor(
        face_2d_entity,
        old_palette_color_entity_opt,
    )];
}
