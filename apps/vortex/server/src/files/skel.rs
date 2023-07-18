use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, SystemState},
};
use naia_bevy_server::{BitReader, BitWriter, CommandsExt, Serde, SerdeErr, Server, UnsignedVariableInteger};

use vortex_proto::components::{FileSystemChild, Vertex3d, VertexChild, VertexRootChild, VertexSerdeInt};

use crate::files::{FileReader, FileWriter};

// Actions
enum SkelAction {
    //////// x,   y,   z, parent_id (0 for none)
    Vertex(i16, i16, i16, Option<u16>),
}

// Writer
pub struct SkelWriter;

impl SkelWriter {
    fn new_default_actions(&self) -> Vec<SkelAction> {
        let mut output = Vec::new();

        // waist
        output.push(SkelAction::Vertex(0, 0, 0, None));

        // neck
        output.push(SkelAction::Vertex(0, 70, 0, Some(0)));

        // head
        output.push(SkelAction::Vertex(0, 90, 0, Some(1)));

        // left arm
        output.push(SkelAction::Vertex(25, 5, 0, Some(1)));

        // right arm
        output.push(SkelAction::Vertex(-25, 5, 0, Some(1)));

        // left leg
        output.push(SkelAction::Vertex(20, -90, 0, Some(0)));

        // right leg
        output.push(SkelAction::Vertex(-20, -90, 0, Some(0)));

        output
    }

    fn world_to_actions(
        &self,
        world: &mut World,
        content_entities: &Vec<Entity>,
    ) -> Vec<SkelAction> {
        let mut system_state: SystemState<(Server, Query<(&Vertex3d, Option<&FileSystemChild>)>)> =
            SystemState::new(world);
        let (server, vertex_query) = system_state.get_mut(world);

        let mut output = Vec::new();

        /////////////////////////////  id,   x,   y,   z, parent_entity   /////////////////
        let mut map: HashMap<Entity, (usize, i16, i16, i16, Option<Entity>)> = HashMap::new();

        for (id, entity) in content_entities.iter().enumerate() {
            let (vertex, has_parent_opt) = vertex_query.get(*entity).unwrap();

            let parent_id: Option<Entity> = {
                match has_parent_opt {
                    Some(has_parent) => match has_parent.parent_id.get(&server) {
                        Some(parent_id) => Some(parent_id),
                        None => None,
                    },
                    None => None,
                }
            };

            map.insert(*entity, (id, vertex.x(), vertex.y(), vertex.z(), parent_id));
        }

        for entity in content_entities.iter() {
            let (_, x, y, z, parent_entity_opt) = map.get(entity).unwrap();
            let parent_id = parent_entity_opt.map(|parent_entity| {
                let (parent_id, _, _, _, _) = map.get(&parent_entity).unwrap();
                *parent_id as u16
            });
            output.push(SkelAction::Vertex(*x, *y, *z, parent_id));
        }

        output
    }

    fn write_from_actions(&self, actions: Vec<SkelAction>) -> Box<[u8]> {
        let mut bit_writer = BitWriter::new();

        for action in actions {
            match action {
                SkelAction::Vertex(x, y, z, parent_id_opt) => {
                    // continue bit
                    true.ser(&mut bit_writer);

                    // encode X, Y, Z
                    VertexSerdeInt::from(x).ser(&mut bit_writer);
                    VertexSerdeInt::from(y).ser(&mut bit_writer);
                    VertexSerdeInt::from(z).ser(&mut bit_writer);
                    let parent_id = {
                        if let Some(parent_id) = parent_id_opt {
                            parent_id + 1
                        } else {
                            0
                        }
                    };
                    UnsignedVariableInteger::<6>::from(parent_id).ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        false.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

impl FileWriter for SkelWriter {
    fn write(&self, world: &mut World, content_entities: &Vec<Entity>) -> Box<[u8]> {
        let actions = self.world_to_actions(world, content_entities);
        self.write_from_actions(actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        let actions = self.new_default_actions();
        self.write_from_actions(actions)
    }
}

// Reader
pub struct SkelReader;

impl SkelReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<SkelAction>, SerdeErr> {
        let mut output = Vec::new();

        // handle empty file
        if bit_reader.bytes_len() == 0 {
            return Ok(output);
        }

        // read loop
        loop {
            let continue_bool = bit_reader.read_bit()?;
            if !continue_bool {
                break;
            }

            // read X, Y, Z
            let x = VertexSerdeInt::de(bit_reader)?.to();
            let y = VertexSerdeInt::de(bit_reader)?.to();
            let z = VertexSerdeInt::de(bit_reader)?.to();
            let parent_id: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
            let parent_id_opt = {
                if parent_id == 0 {
                    None
                } else {
                    Some(parent_id - 1)
                }
            };

            output.push(SkelAction::Vertex(x, y, z, parent_id_opt));
        }
        Ok(output)
    }

    fn actions_to_world(
        commands: &mut Commands,
        server: &mut Server,
        new_entities: &mut Vec<Entity>,
        actions: Vec<SkelAction>,
    ) -> Result<(), SerdeErr> {
        let mut entities: Vec<(Entity, i16, i16, i16, Option<u16>)> = Vec::new();

        for action in actions {
            match action {
                SkelAction::Vertex(x, y, z, parent_id_opt) => {
                    let entity_id = commands
                        .spawn_empty()
                        .enable_replication(server)
                        .id();
                    entities.push((entity_id, x, y, z, parent_id_opt));
                }
            }
        }

        for (entity, x, y, z, parent_id_opt) in entities.iter() {
            let mut entity_mut = commands.entity(*entity);
            entity_mut.insert(Vertex3d::new(*x, *y, *z));

            if let Some(parent_id) = parent_id_opt {
                let (parent_entity, _, _, _, _) = entities.get(*parent_id as usize).unwrap();
                let mut parent_component = VertexChild::new();
                parent_component.parent_id.set(server, parent_entity);
                entity_mut.insert(parent_component);
            } else {
                entity_mut.insert(VertexRootChild);
            }

            new_entities.push(*entity);
        }

        Ok(())
    }
}

impl FileReader for SkelReader {
    fn read(&self, commands: &mut Commands, server: &mut Server, bytes: &Box<[u8]>) -> Vec<Entity> {
        let mut new_entities = Vec::new();
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = Self::read_to_actions(&mut bit_reader) else {
            panic!("Error reading .skel file");
        };

        let Ok(()) = Self::actions_to_world(commands, server, &mut new_entities, actions) else {
            panic!("Error reading .skel file");
        };
        new_entities
    }
}
