use bevy_ecs::{
    prelude::World,
    system::{Commands, SystemState},
};

use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use render_egui::egui::Color32;

use crate::app::resources::{action::palette::PaletteAction, palette_manager::PaletteManager};

pub fn execute(
    world: &mut World,
    palette_manager: &mut PaletteManager,
    action: PaletteAction,
) -> Vec<PaletteAction> {
    let PaletteAction::InsertColor(file_entity, color_index, content_opt) = action else {
        panic!("Expected InsertColor");
    };

    info!(
        "InsertColor({:?}, {:?}, {:?})",
        file_entity, color_index, content_opt
    );

    let last_color_index: usize;

    {
        let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
        let (mut commands, mut client) = system_state.get_mut(world);

        last_color_index = palette_manager.current_color_index();
        info!("current color index: {}", last_color_index);

        let last_color_entity = palette_manager
            .get_color_entity(&file_entity, last_color_index)
            .unwrap();
        commands
            .entity(last_color_entity)
            .release_authority(&mut client);

        let mut init_color = Color32::WHITE;
        if let Some(color) = content_opt {
            init_color = color;
        }
        palette_manager.insert_color(
            &mut commands,
            &mut client,
            file_entity,
            color_index,
            init_color,
        );

        palette_manager.select_color(color_index);

        // TODO: migrate undo/redo entities

        system_state.apply(world);
    }

    return vec![PaletteAction::DeleteColor(file_entity, color_index)];
}
