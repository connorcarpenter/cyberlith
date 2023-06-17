use bevy_ecs::{entity::Entity, prelude::{Commands, World}, system::{Query, SystemState}};
use bevy_log::info;
use naia_bevy_server::{BitReader, BitWriter, Serde, SerdeErr};

use vortex_proto::components::Vertex3d;

use crate::files::{FileReader, FileWriter};

// Actions
enum SkelAction {
    Vertex(u16, u16, u16)
}

// Writer
pub struct SkelWriter;

impl SkelWriter {
    fn write_to_actions(&self, world: &mut World, content_entities: &Vec<Entity>) -> Vec<SkelAction> {
        let mut system_state: SystemState<Query<&Vertex3d>> = SystemState::new(world);
        let vertex_query = system_state.get(world);

        let mut output = Vec::new();
        for entity in content_entities {
            let vertex = vertex_query.get(*entity).unwrap();
            output.push(SkelAction::Vertex(*vertex.x, *vertex.y, *vertex.z));
        }

        output
    }
}

impl FileWriter for SkelWriter {
    fn write(&self, world: &mut World, content_entities: &Vec<Entity>) -> Box<[u8]> {
        let actions = self.write_to_actions(world, content_entities);

        let mut bit_writer = BitWriter::new();

        for action in actions {
            match action {
                SkelAction::Vertex(x, y, z) => {
                    // continue bit
                    true.ser(&mut bit_writer);

                    // encode X, Y, Z
                    x.ser(&mut bit_writer);
                    y.ser(&mut bit_writer);
                    z.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        false.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

// Reader
pub struct SkelReader;

impl SkelReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<SkelAction>, SerdeErr> {
        let mut output = Vec::new();
        loop {
            let continue_bool = bit_reader.read_bit()?;
            if !continue_bool {
                break;
            }

            // read X, Y, Z
            let x = u16::de(bit_reader)?;
            let y = u16::de(bit_reader)?;
            let z = u16::de(bit_reader)?;

            output.push(SkelAction::Vertex(x, y, z));
        }
        Ok(output)
    }

    fn read_to_world(
        commands: &mut Commands,
        bit_reader: &mut BitReader,
        new_entities: &mut Vec<Entity>,
    ) -> Result<(), SerdeErr> {
        let actions = Self::read_to_actions(bit_reader)?;

        for action in actions {
            match action {
                SkelAction::Vertex(x, y, z) => {
                    let entity = commands
                        .spawn_empty()
                        .insert(Vertex3d::new(x, y, z))
                        .id();
                    new_entities.push(entity);
                }
            }
        }

        Ok(())
    }
}

impl FileReader for SkelReader {
    fn read(&self, commands: &mut Commands, bytes: &Box<[u8]>) -> Vec<Entity> {
        let mut new_entities = Vec::new();
        let mut bit_reader = BitReader::new(bytes);
        let result = Self::read_to_world(commands, &mut bit_reader, &mut new_entities);
        if result.is_err() {
            info!("Error reading .skel file");
        }
        new_entities
    }
}