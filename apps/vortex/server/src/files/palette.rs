use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{
    BitReader, CommandsExt, FileBitWriter, ReplicationConfig, Serde, SerdeErr, Server,
};

use vortex_proto::components::PaletteColor;

use crate::{
    files::FileWriter,
    resources::{ContentEntityData, PaletteManager, Project},
};

// Actions
#[derive(Clone)]
enum PaletteAction {
    // red, green, blue
    Color(u8, u8, u8),
}

#[derive(Serde, Clone, PartialEq)]
enum PaletteActionType {
    Color,
    None,
}

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

    fn write_from_actions(&self, actions: Vec<Option<PaletteAction>>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action_opt in actions {
            let action = action_opt.unwrap(); // missing some color is an error
            match action {
                PaletteAction::Color(r, g, b) => {
                    PaletteActionType::Color.ser(&mut bit_writer);
                    r.ser(&mut bit_writer);
                    g.ser(&mut bit_writer);
                    b.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        PaletteActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

impl FileWriter for PaletteWriter {
    fn write(
        &self,
        world: &mut World,
        _project: &Project,
        _content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let actions = self.world_to_actions(world);
        self.write_from_actions(actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        let mut actions = Vec::new();

        actions.push(Some(PaletteAction::Color(255, 255, 255))); // white
        actions.push(Some(PaletteAction::Color(0, 0, 0))); // black
        actions.push(Some(PaletteAction::Color(255, 0, 0))); // red
        actions.push(Some(PaletteAction::Color(0, 255, 0))); // green
        actions.push(Some(PaletteAction::Color(0, 0, 255))); // blue
        actions.push(Some(PaletteAction::Color(255, 255, 0))); // yellow
        actions.push(Some(PaletteAction::Color(0, 255, 255))); // cyan
        actions.push(Some(PaletteAction::Color(255, 0, 255))); // magenta

        self.write_from_actions(actions)
    }
}

// Reader
pub struct PaletteReader;

impl PaletteReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<PaletteAction>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = PaletteActionType::de(bit_reader)?;

            match action_type {
                PaletteActionType::Color => {
                    let r = u8::de(bit_reader)?;
                    let g = u8::de(bit_reader)?;
                    let b = u8::de(bit_reader)?;
                    actions.push(PaletteAction::Color(r, g, b));
                }
                PaletteActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }

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

        let Ok(actions) = Self::read_to_actions(&mut bit_reader) else {
            panic!("Error reading .palette file");
        };

        let result = Self::actions_to_world(world, file_entity, actions);

        result
    }
}
