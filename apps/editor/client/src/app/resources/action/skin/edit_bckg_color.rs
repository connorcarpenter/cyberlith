use bevy_ecs::{
    entity::Entity,
    event::EventWriter,
    prelude::World,
    system::{Commands, Query, ResMut, SystemState},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt};

use editor_proto::components::BackgroundSkinColor;

use crate::app::{
    events::ShapeColorResyncEvent,
    plugin::Main,
    resources::{action::skin::SkinAction, canvas::Canvas, skin_manager::SkinManager},
};

pub(crate) fn execute(
    world: &mut World,
    current_file_entity: Entity,
    action: SkinAction,
) -> Vec<SkinAction> {
    let SkinAction::EditBckgColor(new_palette_color_entity) = action else {
        panic!("Expected EditBckgColor");
    };

    info!("EditBckgColor(new_color: `{:?}`)", new_palette_color_entity);
    let mut system_state: SystemState<(
        Commands,
        Client<Main>,
        ResMut<Canvas>,
        ResMut<SkinManager>,
        EventWriter<ShapeColorResyncEvent>,
        Query<&mut BackgroundSkinColor>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut client,
        mut canvas,
        skin_manager,
        mut shape_color_resync_event_writer,
        mut bckg_color_q,
    ) = system_state.get_mut(world);

    let bckg_color_entity = *skin_manager
        .file_to_bckg_entity(&current_file_entity)
        .unwrap();

    // check that we have auth
    let auth = commands
        .entity(bckg_color_entity)
        .authority(&client)
        .unwrap();
    if auth.is_denied() {
        warn!("EditBckgColor action denied, we do not have auth");
        return vec![];
    }
    let mut must_request_auth = true;
    if auth.is_requested() || auth.is_granted() {
        must_request_auth = false;
    }
    if must_request_auth {
        commands
            .entity(bckg_color_entity)
            .request_authority(&mut client);
    }

    // get palette color entity
    let Ok(mut bckg_color) = bckg_color_q.get_mut(bckg_color_entity) else {
        panic!(
            "Failed to get BackgroundSkinColor for face color entity {:?}!",
            bckg_color_entity
        );
    };
    let old_palette_color_entity = bckg_color.palette_color_entity.get(&client).unwrap();

    // set palette color entity
    bckg_color
        .palette_color_entity
        .set(&client, &new_palette_color_entity);

    canvas.queue_resync_shapes();
    shape_color_resync_event_writer.send(ShapeColorResyncEvent);

    system_state.apply(world);

    return vec![SkinAction::EditBckgColor(old_palette_color_entity)];
}
