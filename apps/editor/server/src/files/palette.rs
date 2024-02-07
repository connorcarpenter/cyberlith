use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{BitReader, CommandsExt, ReplicationConfig, Server};

use asset_io::PaletteAction;

use editor_proto::components::PaletteColor;

use crate::{
    files::FileWriter,
    resources::{ContentEntityData, PaletteManager, Project},
};

// Writer
pub struct PaletteWriter;

impl PaletteWriter {
    fn world_to_actions(&self, world: &mut World) -> Vec<Option<PaletteAction>> {
        let mut system_state: SystemState<Query<&PaletteColor>> = SystemState::new(world);
        let color_q = system_state.get_mut(world);

        let mut actions = Vec::new();

        for color in color_q.iter() {
            let index = *color.index as usize;
            let r = *color.r;
            let g = *color.g;
            let b = *color.b;

            if index >= actions.len() {
                actions.resize(index + 1, None);
            }

            actions[index] = Some(PaletteAction::Color(r, g, b));
        }

        actions
    }
}

impl FileWriter for PaletteWriter {
    fn write(
        &self,
        world: &mut World,
        _project: &Project,
        _content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let mut action_index = 0;
        let actions = self.world_to_actions(world);
        let mut output_actions = Vec::new();
        for action_opt in actions {
            let Some(action) = action_opt else {
                panic!("Palette action is missing! index: {}", action_index);
            };
            output_actions.push(action);
            action_index += 1;
        }
        PaletteAction::write(output_actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        let mut actions = Vec::new();

        actions.push(PaletteAction::Color(255, 255, 255)); // white
        actions.push(PaletteAction::Color(0, 0, 0)); // black
        actions.push(PaletteAction::Color(255, 0, 0)); // red
        actions.push(PaletteAction::Color(0, 255, 0)); // green
        actions.push(PaletteAction::Color(0, 0, 255)); // blue
        actions.push(PaletteAction::Color(255, 255, 0)); // yellow
        actions.push(PaletteAction::Color(0, 255, 255)); // cyan
        actions.push(PaletteAction::Color(255, 0, 255)); // magenta

        PaletteAction::write(actions)
    }
}

// Reader
pub struct PaletteReader;

impl PaletteReader {
    fn actions_to_world(
        world: &mut World,
        file_entity: &Entity,
        actions: Vec<PaletteAction>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut output = HashMap::new();
        let mut index = 0;

        let mut system_state: SystemState<(Commands, Server, ResMut<PaletteManager>)> =
            SystemState::new(world);
        let (mut commands, mut server, mut palette_manager) = system_state.get_mut(world);

        for action in actions {
            match action {
                PaletteAction::Color(r, g, b) => {
                    let mut color_component = PaletteColor::new(index, r, g, b);
                    color_component
                        .owning_file_entity
                        .set(&mut server, file_entity);
                    let entity_id = commands
                        .spawn_empty()
                        .enable_replication(&mut server)
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(color_component)
                        .id();
                    info!(
                        "palette color entity: `{:?}`, rgb:({}, {}, {})",
                        entity_id, r, g, b
                    );
                    output.insert(entity_id, ContentEntityData::new_palette_color());

                    palette_manager.on_create_color(&file_entity, &entity_id, index as usize, None);
                }
            }

            index += 1;
        }

        system_state.apply(world);

        output
    }

    pub fn read(
        &self,
        world: &mut World,
        file_entity: &Entity,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = PaletteAction::read(&mut bit_reader) else {
            panic!("Error reading .palette file");
        };

        let result = Self::actions_to_world(world, file_entity, actions);

        result
    }
}
