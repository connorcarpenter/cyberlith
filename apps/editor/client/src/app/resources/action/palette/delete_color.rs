use bevy_ecs::{
    prelude::World,
    system::{Commands, Query, SystemState},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use render_egui::egui::Color32;

use editor_proto::components::PaletteColor;

use crate::app::{resources::{action::palette::PaletteAction, palette_manager::PaletteManager}, plugin::Main};

pub fn execute(
    world: &mut World,
    palette_manager: &mut PaletteManager,
    action: PaletteAction,
) -> Vec<PaletteAction> {
    let PaletteAction::DeleteColor(file_entity, color_index) = action else {
        panic!("Expected DeleteColor");
    };

    info!("DeleteColor({:?}, {:?})", file_entity, color_index);

    let mut system_state: SystemState<(Commands, Client<Main>, Query<&PaletteColor>)> =
        SystemState::new(world);
    let (mut commands, mut client, color_q) = system_state.get_mut(world);

    let color_entity = palette_manager
        .get_color_entity(&file_entity, color_index)
        .unwrap();

    // check auth
    if let Some(auth) = commands.entity(color_entity).authority(&client) {
        if !auth.is_requested() && !auth.is_granted() {
            panic!(
                "current color entity `{:?}` does not have auth!",
                color_entity
            );
        }
    }

    let color_stored;
    let Ok(color) = color_q.get(color_entity) else {
        panic!("Expected color");
    };
    color_stored = Color32::from_rgb(*color.r, *color.g, *color.b);

    // despawn
    commands.entity(color_entity).despawn();

    // deregister
    palette_manager.deregister_color(&file_entity, &color_entity, color_index);

    // select frame - 1
    if color_index > 0 {
        let next_color_index = color_index - 1;
        let next_color_entity = palette_manager
            .get_color_entity(&file_entity, next_color_index)
            .unwrap();
        commands
            .entity(next_color_entity)
            .request_authority(&mut client);
        palette_manager.select_color(next_color_index);
    }

    system_state.apply(world);

    return vec![PaletteAction::InsertColor(
        file_entity,
        color_index,
        Some(color_stored),
    )];
}
