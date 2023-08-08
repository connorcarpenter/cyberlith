use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{
    BitReader, BitWriter, CommandsExt, ReplicationConfig, Serde, SerdeErr, Server,
    UnsignedVariableInteger,
};

use vortex_proto::components::{
    Vertex3d, VertexChild, VertexRoot, VertexSerdeInt,
};

use crate::{
    files::{FileReadOutput, FileReader, FileWriter},
    resources::VertexManager,
};

// Actions
#[derive(Debug)]
enum SkelAction {
    //////// x,   y,   z, parent_id (0 for none)
    Vertex(i16, i16, i16, Option<u16>),
}

// Writer
pub struct SkelWriter;

impl SkelWriter {
    fn new_default_actions(&self) -> Vec<SkelAction> {
        let mut output = Vec::new();

        output.push(SkelAction::Vertex(0, 0, 0, None));

        output
    }

    fn world_to_actions(
        &self,
        world: &mut World,
        content_entities: &Vec<Entity>,
    ) -> Vec<SkelAction> {
        let mut system_state: SystemState<(
            Server,
            Query<(&Vertex3d, Option<&VertexChild>)>,
        )> = SystemState::new(world);
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

            let vertex_info = (id, vertex.x(), vertex.y(), vertex.z(), parent_id);
            map.insert(*entity, vertex_info);
        }

        for entity in content_entities.iter() {
            let (_, x, y, z, parent_entity_opt) = map.get(entity).unwrap();
            let parent_id = parent_entity_opt.map(|parent_entity| {
                let (parent_id, _, _, _, _) = map.get(&parent_entity).unwrap();
                *parent_id as u16
            });
            let vertex_info = SkelAction::Vertex(*x, *y, *z, parent_id);
            output.push(vertex_info);
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
    fn write(&self, world: &mut World, content_entities: &HashSet<Entity>) -> Box<[u8]> {
        let content_entities_vec: Vec<Entity> = content_entities.iter().map(|e| *e).collect();
        let actions = self.world_to_actions(world, &content_entities_vec);
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
        actions: Vec<SkelAction>,
    ) -> Result<FileReadOutput, SerdeErr> {
        let mut output = Vec::new();

        let mut entities: Vec<(Entity, i16, i16, i16, Option<u16>)> = Vec::new();

        for action in actions {
            match action {
                SkelAction::Vertex(x, y, z, parent_id_opt) => {
                    let entity_id = commands.spawn_empty().enable_replication(server).id();
                    info!("spawning vertex entity {:?}", entity_id);
                    if parent_id_opt.is_some() {
                        commands
                            .entity(entity_id)
                            .configure_replication(ReplicationConfig::Delegated);
                        entities.push((entity_id, x, y, z, parent_id_opt));
                    } else {
                        // root node should always be at 0,0,0 ... you can refactor these files later
                        entities.push((entity_id, 0, 0, 0, parent_id_opt));
                    }
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
                output.push((*entity, Some(*parent_entity)));
            } else {
                entity_mut.insert(VertexRoot);
                output.push((*entity, None));
            }
        }

        Ok(FileReadOutput::Skel(output))
    }

    pub fn post_process_entities(
        vertex_manager: &mut VertexManager,
        entities: Vec<(Entity, Option<Entity>)>,
    ) -> HashSet<Entity> {
        let mut new_content_entities = HashSet::new();

        for (entity, parent_opt) in entities {
            new_content_entities.insert(entity);
            vertex_manager.on_create_vertex(&entity, parent_opt);
        }
        vertex_manager.finalize_vertex_creation();

        new_content_entities
    }
}

impl FileReader for SkelReader {
    fn read(
        &self,
        commands: &mut Commands,
        server: &mut Server,
        bytes: &Box<[u8]>,
    ) -> FileReadOutput {
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = Self::read_to_actions(&mut bit_reader) else {
            panic!("Error reading .skel file");
        };

        let Ok(result) = Self::actions_to_world(commands, server, actions) else {
            panic!("Error reading .skel file");
        };

        result
    }
}
