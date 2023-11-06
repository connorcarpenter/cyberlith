use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, Res, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{
    BitReader, CommandsExt, FileBitWriter, ReplicationConfig, Serde, SerdeErr, Server,
    UnsignedVariableInteger,
};

use vortex_proto::components::{IconEdge, IconFace, IconVertex, VertexSerdeInt};

use crate::{
    files::{FileWriter, ShapeTypeData},
    resources::{IconManager, ContentEntityData, Project},
};

// Actions
#[derive(Debug, Clone)]
enum IconAction {
    //////// x,   y//
    Vertex(i16, i16),
    //// id1, id2 // (vertex ids)
    Edge(u16, u16),
    //// order_index, id1, id2, id3 // (vertex ids) // id4, id5, id6 (edge ids)
    Face(u16, u16, u16, u16, u16, u16, u16),
}

#[derive(Serde, Clone, PartialEq)]
enum IconActionType {
    None,
    Vertex,
    Edge,
    Face,
}

// Writer
pub struct IconWriter;

impl IconWriter {
    fn world_to_actions(
        &self,
        world: &mut World,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Vec<IconAction> {
        let content_entities = content_entities.keys().cloned().collect::<Vec<Entity>>();

        let mut system_state: SystemState<(
            Server,
            Res<IconManager>,
            Query<&IconVertex>,
            Query<&IconEdge>,
            Query<&IconFace>,
        )> = SystemState::new(world);
        let (server, icon_manager, vertex_q, edge_q, face_q) =
            system_state.get_mut(world);

        let mut output = Vec::new();

        /////////////////////////////////////  id /////////////////
        let mut vertex_map: HashMap<Entity, usize> = HashMap::new();
        let mut edge_map: HashMap<Entity, usize> = HashMap::new();
        let mut face_list: Vec<Option<IconAction>> = Vec::new();

        info!(
            "writing in world_to_actions(), content_entities: `{:?}`",
            content_entities
        );

        let mut vertex_count: usize = 0;
        for entity in content_entities.iter() {
            if let Ok(vertex) = vertex_q.get(*entity) {
                // entity is a vertex
                vertex_map.insert(*entity, vertex_count);
                let vertex_info = IconAction::Vertex(vertex.x(), vertex.y());
                output.push(vertex_info);
                vertex_count += 1;
            }
        }

        let mut edge_count: usize = 0;
        for entity in content_entities.iter() {
            if vertex_map.contains_key(entity) {
                // entity is a vertex
                continue;
            }

            if let Ok(edge) = edge_q.get(*entity) {
                // entity is an edge
                edge_map.insert(*entity, edge_count);
                let vertex_a_entity = edge.start.get(&server).unwrap();
                let vertex_b_entity = edge.end.get(&server).unwrap();
                let vertex_a_id = *vertex_map.get(&vertex_a_entity).unwrap();
                let vertex_b_id = *vertex_map.get(&vertex_b_entity).unwrap();
                let edge_info = IconAction::Edge(vertex_a_id as u16, vertex_b_id as u16);
                output.push(edge_info);
                edge_count += 1;
            }
        }

        for entity in content_entities.iter() {
            if vertex_map.contains_key(entity) {
                // entity is a vertex
                continue;
            }
            if edge_map.contains_key(entity) {
                // entity is an edge
                continue;
            }

            if let Ok(face) = face_q.get(*entity) {
                let Some(face_index) = icon_manager.get_face_index(entity) else {
                    panic!("face entity {:?} does not have an index!", entity);
                };

                // entity is a face
                let vertex_a_entity = face.vertex_a.get(&server).unwrap();
                let vertex_b_entity = face.vertex_b.get(&server).unwrap();
                let vertex_c_entity = face.vertex_c.get(&server).unwrap();
                let vertex_a_id = *vertex_map.get(&vertex_a_entity).unwrap();
                let vertex_b_id = *vertex_map.get(&vertex_b_entity).unwrap();
                let vertex_c_id = *vertex_map.get(&vertex_c_entity).unwrap();

                let edge_a_entity = face.edge_a.get(&server).unwrap();
                let edge_b_entity = face.edge_b.get(&server).unwrap();
                let edge_c_entity = face.edge_c.get(&server).unwrap();
                let edge_a_id = *edge_map.get(&edge_a_entity).unwrap();
                let edge_b_id = *edge_map.get(&edge_b_entity).unwrap();
                let edge_c_id = *edge_map.get(&edge_c_entity).unwrap();

                let face_info = IconAction::Face(
                    face_index as u16,
                    vertex_a_id as u16,
                    vertex_b_id as u16,
                    vertex_c_id as u16,
                    edge_a_id as u16,
                    edge_b_id as u16,
                    edge_c_id as u16,
                );
                if face_index >= face_list.len() {
                    face_list.resize(face_index + 1, None);
                }
                face_list[face_index] = Some(face_info);
            } else {
                panic!("entity is not a vertex, edge, or face");
            }
        }

        for face_info_opt in face_list {
            let Some(face_info) = face_info_opt else {
                panic!("face_list contains None");
            };
            output.push(face_info);
        }

        output
    }

    fn write_from_actions(&self, actions: Vec<IconAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        let mut test_face_index = 0;
        for (action_id, action) in actions.iter().enumerate() {
            match action {
                IconAction::Vertex(x, y) => {
                    // continue bit
                    IconActionType::Vertex.ser(&mut bit_writer);

                    // encode X, Y
                    VertexSerdeInt::from(*x).ser(&mut bit_writer);
                    VertexSerdeInt::from(*y).ser(&mut bit_writer);

                    info!("writing vertex {} : ({}, {})", action_id, x, y);
                }
                IconAction::Edge(vertex_a, vertex_b) => {
                    // continue bit
                    IconActionType::Edge.ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::from(*vertex_a).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*vertex_b).ser(&mut bit_writer);

                    info!("writing edge : ({}, {})", vertex_a, vertex_b);
                }
                IconAction::Face(
                    face_index,
                    vertex_a,
                    vertex_b,
                    vertex_c,
                    edge_a,
                    edge_b,
                    edge_c,
                ) => {
                    if *face_index != test_face_index {
                        panic!(
                            "face_index {:?} does not match test_face_index {:?}",
                            face_index, test_face_index
                        );
                    }

                    // continue bit
                    IconActionType::Face.ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::from(*vertex_a).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*vertex_b).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*vertex_c).ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::from(*edge_a).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*edge_b).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*edge_c).ser(&mut bit_writer);

                    info!(
                        "writing face : ({}, {}, {}, {}, {}, {})",
                        vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c
                    );

                    test_face_index += 1;
                }
            }
        }

        // continue bit
        IconActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

impl FileWriter for IconWriter {
    fn write(
        &self,
        world: &mut World,
        _project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let actions = self.world_to_actions(world, content_entities);
        self.write_from_actions(actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        let mut default_actions = Vec::new();

        default_actions.push(IconAction::Vertex(0, 0));

        self.write_from_actions(default_actions)
    }
}

// Reader
pub struct IconReader;

impl IconReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<IconAction>, SerdeErr> {
        let mut output = Vec::new();

        // handle empty file
        if bit_reader.bytes_len() == 0 {
            return Ok(output);
        }

        let mut face_index = 0;

        // read loop
        loop {
            let continue_type = IconActionType::de(bit_reader)?;

            match continue_type {
                IconActionType::None => break,
                IconActionType::Vertex => {
                    // read X, Y
                    let x = VertexSerdeInt::de(bit_reader)?.to();
                    let y = VertexSerdeInt::de(bit_reader)?.to();

                    output.push(IconAction::Vertex(x, y));
                }
                IconActionType::Edge => {
                    let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    output.push(IconAction::Edge(vertex_a, vertex_b));
                }
                IconActionType::Face => {
                    let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    let edge_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let edge_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let edge_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    output.push(IconAction::Face(
                        face_index, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c,
                    ));

                    face_index += 1;
                }
            }
        }
        Ok(output)
    }

    fn actions_to_world(
        world: &mut World,
        file_entity: &Entity,
        actions: Vec<IconAction>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut system_state: SystemState<(Commands, Server, ResMut<IconManager>)> =
            SystemState::new(world);
        let (mut commands, mut server, mut icon_manager) = system_state.get_mut(world);

        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut output = Vec::new();

        for action in actions {
            match action {
                IconAction::Vertex(x, y) => {
                    let entity_id = commands
                        .spawn_empty()
                        .enable_replication(&mut server)
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(IconVertex::new(x, y))
                        .id();
                    info!("spawning icon vertex entity {:?}", entity_id);
                    vertices.push(entity_id);
                    output.push((entity_id, ShapeTypeData::Vertex));
                }
                IconAction::Edge(vertex_a_index, vertex_b_index) => {
                    let Some(vertex_a_entity) = vertices.get(vertex_a_index as usize) else {
                        panic!("edge's vertex_a_index is `{:?}` and list of vertices is `{:?}`", vertex_a_index, vertices);
                    };
                    let Some(vertex_b_entity) = vertices.get(vertex_b_index as usize) else {
                        panic!("edge's vertex_b_index is `{:?}` and list of vertices is `{:?}`", vertex_b_index, vertices);
                    };

                    let mut edge_component = IconEdge::new();
                    edge_component.start.set(&server, vertex_a_entity);
                    edge_component.end.set(&server, vertex_b_entity);

                    let entity_id = commands
                        .spawn_empty()
                        .enable_replication(&mut server)
                        // setting to Delegated to match client-created edges
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(edge_component)
                        .id();
                    info!("spawning mesh edge entity {:?}", entity_id);
                    edges.push(entity_id);
                    output.push((
                        entity_id,
                        ShapeTypeData::Edge(*vertex_a_entity, *vertex_b_entity),
                    ));
                }
                IconAction::Face(
                    face_index,
                    vertex_a_index,
                    vertex_b_index,
                    vertex_c_index,
                    edge_a_index,
                    edge_b_index,
                    edge_c_index,
                ) => {
                    let vertex_a_entity = *vertices.get(vertex_a_index as usize).unwrap();
                    let vertex_b_entity = *vertices.get(vertex_b_index as usize).unwrap();
                    let vertex_c_entity = *vertices.get(vertex_c_index as usize).unwrap();

                    let edge_a_entity = *edges.get(edge_a_index as usize).unwrap();
                    let edge_b_entity = *edges.get(edge_b_index as usize).unwrap();
                    let edge_c_entity = *edges.get(edge_c_index as usize).unwrap();

                    let mut face_component = IconFace::new();
                    face_component.vertex_a.set(&server, &vertex_a_entity);
                    face_component.vertex_b.set(&server, &vertex_b_entity);
                    face_component.vertex_c.set(&server, &vertex_c_entity);
                    face_component.edge_a.set(&server, &edge_a_entity);
                    face_component.edge_b.set(&server, &edge_b_entity);
                    face_component.edge_c.set(&server, &edge_c_entity);

                    let entity_id = commands
                        .spawn_empty()
                        .enable_replication(&mut server)
                        // setting to Delegated to match client-created faces
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(face_component)
                        .id();
                    info!(
                        "spawning icon face entity `{:?}`, index is {:?}",
                        entity_id, face_index
                    );
                    output.push((
                        entity_id,
                        ShapeTypeData::Face(
                            face_index as usize,
                            vertex_a_entity,
                            vertex_b_entity,
                            vertex_c_entity,
                        ),
                    ));
                }
            }
        }

        let output = IconReader::post_process_entities(&mut icon_manager, file_entity, output);

        system_state.apply(world);

        output
    }
}

impl IconReader {
    pub fn read(
        &self,
        world: &mut World,
        file_entity: &Entity,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = Self::read_to_actions(&mut bit_reader) else {
            panic!("Error reading .icon file");
        };

        let result = Self::actions_to_world(world, file_entity, actions);

        result
    }
}

impl IconReader {
    pub fn post_process_entities(
        icon_manager: &mut IconManager,
        file_entity: &Entity,
        shape_entities: Vec<(Entity, ShapeTypeData)>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut new_content_entities = HashMap::new();

        for (entity, shape_type_data) in shape_entities {
            new_content_entities
                .insert(entity, ContentEntityData::new_icon_shape(shape_type_data.into()));

            match shape_type_data {
                ShapeTypeData::Vertex => {
                    icon_manager.on_create_vertex(entity);
                }
                ShapeTypeData::Edge(start, end) => {
                    icon_manager.on_create_edge(start, entity, end);
                }
                ShapeTypeData::Face(index, vert_a, vert_b, vert_c) => {
                    icon_manager.on_create_face(
                        file_entity,
                        Some(index),
                        entity,
                        vert_a,
                        vert_b,
                        vert_c,
                    );
                }
            }
        }

        new_content_entities
    }
}
