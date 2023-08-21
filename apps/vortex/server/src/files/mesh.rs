use std::collections::HashMap;

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

use vortex_proto::components::{Edge3d, Face3d, FileType, FileTypeValue, Vertex3d, VertexSerdeInt};

use crate::{
    files::{ShapeTypeData, FileReadOutput, FileReader, FileWriter},
    resources::{ContentEntityData, ShapeManager},
};

// Actions
#[derive(Debug)]
enum MeshAction {
    //////// x,   y,   z //
    Vertex(i16, i16, i16),
    //// id1, id2 // (vertex ids)
    Edge(u16, u16),
    //// id1, id2, id3 // (vertex ids)
    Face(u16, u16, u16),
}

#[derive(Serde, Clone, PartialEq)]
enum MeshActionType {
    None,
    Vertex,
    Edge,
    Face,
}

// Writer
pub struct MeshWriter;

impl MeshWriter {
    fn world_to_actions(
        &self,
        world: &mut World,
        content_entities: &Vec<Entity>,
    ) -> Vec<MeshAction> {
        let mut system_state: SystemState<(
            Server,
            Query<&Vertex3d>,
            Query<&Edge3d>,
            Query<&Face3d>,
            Query<&FileType>,
        )> = SystemState::new(world);
        let (server, vertex_q, edge_q, face_q, file_type_q) = system_state.get_mut(world);

        let mut output = Vec::new();

        /////////////////////////////////////  id /////////////////
        let mut vertex_map: HashMap<Entity, usize> = HashMap::new();

        info!("writing in world_to_actions(), content_entities: `{:?}`", content_entities);

        let mut vertex_count: usize = 0;
        for entity in content_entities.iter() {
            let Ok(file_type) = file_type_q.get(*entity) else {
                panic!("entity {:?} does not have a FileType component!", entity);
            };
            if *file_type.value != FileTypeValue::Mesh {
                panic!("entity {:?} does not have a FileType component with value Mesh!", entity);
            }
            if let Ok(vertex) = vertex_q.get(*entity) {
                // entity is a vertex
                vertex_map.insert(*entity, vertex_count);
                let vertex_info = MeshAction::Vertex(vertex.x(), vertex.y(), vertex.z());
                output.push(vertex_info);
                vertex_count += 1;
            }
        }

        for entity in content_entities.iter() {
            if vertex_map.contains_key(entity) {
                // entity is a vertex
                continue;
            }

            if let Ok(edge) = edge_q.get(*entity) {
                // entity is an edge
                let vertex_a_entity = edge.start.get(&server).unwrap();
                let vertex_b_entity = edge.end.get(&server).unwrap();
                let vertex_a_id = *vertex_map.get(&vertex_a_entity).unwrap();
                let vertex_b_id = *vertex_map.get(&vertex_b_entity).unwrap();
                let edge_info = MeshAction::Edge(vertex_a_id as u16, vertex_b_id as u16);
                output.push(edge_info);
            } else if let Ok(face) = face_q.get(*entity) {
                // entity is a face
                let vertex_a_entity = face.vertex_a.get(&server).unwrap();
                let vertex_b_entity = face.vertex_b.get(&server).unwrap();
                let vertex_c_entity = face.vertex_c.get(&server).unwrap();
                let vertex_a_id = *vertex_map.get(&vertex_a_entity).unwrap();
                let vertex_b_id = *vertex_map.get(&vertex_b_entity).unwrap();
                let vertex_c_id = *vertex_map.get(&vertex_c_entity).unwrap();
                let face_info =
                    MeshAction::Face(vertex_a_id as u16, vertex_b_id as u16, vertex_c_id as u16);
                output.push(face_info);
            } else {
                panic!("entity is not a vertex, edge, or face");
            }
        }

        output
    }

    fn write_from_actions(&self, actions: Vec<MeshAction>) -> Box<[u8]> {
        let mut bit_writer = BitWriter::new();

        for (action_id, action) in actions.iter().enumerate() {
            match action {
                MeshAction::Vertex(x, y, z) => {
                    // continue bit
                    MeshActionType::Vertex.ser(&mut bit_writer);

                    // encode X, Y, Z
                    VertexSerdeInt::from(*x).ser(&mut bit_writer);
                    VertexSerdeInt::from(*y).ser(&mut bit_writer);
                    VertexSerdeInt::from(*z).ser(&mut bit_writer);

                    info!("writing vertex {} : ({}, {}, {})", action_id, x, y, z);
                }
                MeshAction::Edge(vertex_a, vertex_b) => {
                    // continue bit
                    MeshActionType::Edge.ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::from(*vertex_a).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*vertex_b).ser(&mut bit_writer);

                    info!("writing edge : ({}, {})", vertex_a, vertex_b);
                }
                MeshAction::Face(vertex_a, vertex_b, vertex_c) => {
                    // continue bit
                    MeshActionType::Face.ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::from(*vertex_a).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*vertex_b).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*vertex_c).ser(&mut bit_writer);

                    info!("writing face : ({}, {}, {})", vertex_a, vertex_b, vertex_c);
                }
            }
        }

        // continue bit
        MeshActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

impl FileWriter for MeshWriter {
    fn write(
        &self,
        world: &mut World,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let content_entities_vec: Vec<Entity> = content_entities.iter().map(|(e, d)| *e).collect();
        let actions = self.world_to_actions(world, &content_entities_vec);
        self.write_from_actions(actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        let mut default_actions = Vec::new();

        default_actions.push(MeshAction::Vertex(0, 0, 0));

        self.write_from_actions(default_actions)
    }
}

// Reader
pub struct MeshReader;

impl MeshReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<MeshAction>, SerdeErr> {
        let mut output = Vec::new();

        // handle empty file
        if bit_reader.bytes_len() == 0 {
            return Ok(output);
        }

        // read loop
        loop {
            let continue_type = MeshActionType::de(bit_reader)?;

            match continue_type {
                MeshActionType::None => break,
                MeshActionType::Vertex => {
                    // read X, Y, Z
                    let x = VertexSerdeInt::de(bit_reader)?.to();
                    let y = VertexSerdeInt::de(bit_reader)?.to();
                    let z = VertexSerdeInt::de(bit_reader)?.to();

                    output.push(MeshAction::Vertex(x, y, z));
                }
                MeshActionType::Edge => {
                    let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    output.push(MeshAction::Edge(vertex_a, vertex_b));
                }
                MeshActionType::Face => {
                    let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    output.push(MeshAction::Face(vertex_a, vertex_b, vertex_c));
                }
            }
        }
        Ok(output)
    }

    fn actions_to_world(
        commands: &mut Commands,
        server: &mut Server,
        actions: Vec<MeshAction>,
    ) -> Result<FileReadOutput, SerdeErr> {
        let mut vertices = Vec::new();
        let mut output = Vec::new();

        for action in actions {
            match action {
                MeshAction::Vertex(x, y, z) => {
                    let entity_id = commands
                        .spawn_empty()
                        .enable_replication(server)
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(Vertex3d::new(x, y, z))
                        .id();
                    info!("spawning mesh vertex entity {:?}", entity_id);
                    vertices.push(entity_id);
                    output.push((entity_id, ShapeTypeData::Vertex));
                }
                MeshAction::Edge(vertex_a_index, vertex_b_index) => {
                    let Some(vertex_a_entity) = vertices.get(vertex_a_index as usize) else {
                        panic!("edge's vertex_a_index is `{:?}` and list of vertices is `{:?}`", vertex_a_index, vertices);
                    };
                    let Some(vertex_b_entity) = vertices.get(vertex_b_index as usize) else {
                        panic!("edge's vertex_b_index is `{:?}` and list of vertices is `{:?}`", vertex_b_index, vertices);
                    };

                    let mut edge_component = Edge3d::new();
                    edge_component.start.set(server, vertex_a_entity);
                    edge_component.end.set(server, vertex_b_entity);

                    let entity_id = commands
                        .spawn_empty()
                        .enable_replication(server)
                        // setting to Delegated to match client-created edges
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(edge_component)
                        .id();
                    info!("spawning mesh edge entity {:?}", entity_id);
                    output.push((entity_id, ShapeTypeData::Edge(*vertex_a_entity, *vertex_b_entity)));
                }
                MeshAction::Face(vertex_a_index, vertex_b_index, vertex_c_index) => {
                    let vertex_a_entity = *vertices.get(vertex_a_index as usize).unwrap();
                    let vertex_b_entity = *vertices.get(vertex_b_index as usize).unwrap();
                    let vertex_c_entity = *vertices.get(vertex_c_index as usize).unwrap();

                    let mut face_component = Face3d::new();
                    face_component.vertex_a.set(server, &vertex_a_entity);
                    face_component.vertex_b.set(server, &vertex_b_entity);
                    face_component.vertex_c.set(server, &vertex_c_entity);

                    let entity_id = commands
                        .spawn_empty()
                        .enable_replication(server)
                        // setting to Delegated to match client-created faces
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(face_component)
                        .id();
                    info!("spawning mesh face entity {:?}", entity_id);
                    output.push((entity_id, ShapeTypeData::Face));
                }
            }
        }

        Ok(FileReadOutput::Mesh(output))
    }
}

impl FileReader for MeshReader {
    fn read(
        &self,
        commands: &mut Commands,
        server: &mut Server,
        bytes: &Box<[u8]>,
    ) -> FileReadOutput {
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = Self::read_to_actions(&mut bit_reader) else {
            panic!("Error reading .mesh file");
        };

        let Ok(result) = Self::actions_to_world(commands, server, actions) else {
            panic!("Error reading .mesh file");
        };

        result
    }
}

impl MeshReader {

    pub fn post_process_entities(
        vertex_manager: &mut ShapeManager,
        shape_entities: Vec<(Entity, ShapeTypeData)>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut new_content_entities = HashMap::new();

        for (entity, shape_type_data) in shape_entities {
            new_content_entities.insert(entity, ContentEntityData::new(shape_type_data.into()));

            match shape_type_data {
                ShapeTypeData::Vertex => {
                    vertex_manager.on_create_mesh_vertex(entity);
                }
                ShapeTypeData::Edge(start, end) => {
                    vertex_manager.on_create_mesh_edge(start, entity, end);
                }
                ShapeTypeData::Face => {}
            }
        }

        new_content_entities
    }
}
