use bevy_ecs::{entity::Entity, prelude::{Commands, World}, system::{Query, SystemState}};
use bevy_log::info;
use naia_bevy_server::{BitReader, BitWriter, Serde, SerdeErr};

use vortex_proto::components::Vertex3d;

use crate::files::{FileReader, FileWriter};

// Writer
pub struct SkelWriter;

impl FileWriter for SkelWriter {
    fn write(&self, world: &mut World, content_entities: &Vec<Entity>) -> Box<[u8]> {
        let mut system_state: SystemState<Query<&Vertex3d>> = SystemState::new(world);
        let vertex_query = system_state.get(world);

        let mut bit_writer = BitWriter::new();

        for entity in content_entities {

            // continue bit
            true.ser(&mut bit_writer);

            // encode X, Y, Z
            let vertex = vertex_query.get(*entity).unwrap();
            vertex.x.ser(&mut bit_writer);
            vertex.y.ser(&mut bit_writer);
            vertex.z.ser(&mut bit_writer);
        }

        false.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

// Reader
pub struct SkelReader;

impl SkelReader {
    fn read_inner(
        &self,
        commands: &mut Commands,
        bit_reader: &mut BitReader,
        new_entities: &mut Vec<Entity>,
    ) -> Result<(), SerdeErr> {
        loop {
            let continue_bool = bit_reader.read_bit()?;
            if !continue_bool {
                break;
            }

            // read X, Y, Z
            let x = u16::de(bit_reader)?;
            let y = u16::de(bit_reader)?;
            let z = u16::de(bit_reader)?;

            let entity = commands
                .spawn_empty()
                .insert(Vertex3d::new(x, y, z))
                .id();

            new_entities.push(entity);
        }

        Ok(())
    }
}

impl FileReader for SkelReader {
    fn read(&self, commands: &mut Commands, bytes: &Box<[u8]>) -> Vec<Entity> {
        let mut new_entities = Vec::new();
        let mut bit_reader = BitReader::new(bytes);
        let result = self.read_inner(commands, &mut bit_reader, &mut new_entities);
        if result.is_err() {
            info!("Error reading skel file");
        }
        new_entities
    }
}