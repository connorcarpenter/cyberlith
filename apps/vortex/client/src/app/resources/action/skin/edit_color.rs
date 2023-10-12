use bevy_ecs::{
    prelude::World,
    system::{Commands, Query, Res, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_client::Client;

use vortex_proto::components::FaceColor;

use crate::app::resources::{
    action::skin::SkinAction, canvas::Canvas, face_manager::FaceManager, skin_manager::SkinManager,
};

pub(crate) fn execute(world: &mut World, action: SkinAction) -> Vec<SkinAction> {
    let SkinAction::EditColor(face_2d_entity, old_palette_color_opt, new_palette_color_opt) = action else {
        panic!("Expected EditColor");
    };

    info!(
        "EditColor(face_2d_entity: `{:?}`, old_color_opt: `{:?}`, new_color_opt: `{:?}`)",
        face_2d_entity, old_palette_color_opt, new_palette_color_opt
    );
    let mut system_state: SystemState<(
        Commands,
        Client,
        ResMut<Canvas>,
        Res<FaceManager>,
        ResMut<SkinManager>,
        Query<&mut FaceColor>,
    )> = SystemState::new(world);
    let (mut commands, client, mut canvas, face_manager, mut skin_manager, mut face_color_q) =
        system_state.get_mut(world);

    let face_3d_entity = face_manager.face_entity_2d_to_3d(&face_2d_entity).unwrap();
    let face_color_entity = *skin_manager.face_to_color_entity(&face_3d_entity).unwrap();

    if let Some(new_palette_color_entity) = new_palette_color_opt {
        let Ok(mut face_color) = face_color_q.get_mut(face_color_entity) else {
            panic!("Failed to get FaceColor for face 3d entity {:?}!", face_3d_entity);
        };
        face_color
            .palette_color_entity
            .set(&client, &new_palette_color_entity);
    } else {
        if let Some(_old_palette_color_entity) = old_palette_color_opt {
            // despawn face color
            commands.entity(face_color_entity).despawn();
            skin_manager.deregister_face_color(&face_color_entity);
        }
    }

    canvas.queue_resync_shapes();

    system_state.apply(world);

    return vec![SkinAction::EditColor(
        face_2d_entity,
        new_palette_color_opt,
        old_palette_color_opt,
    )];
}
