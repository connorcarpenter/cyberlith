use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, Res, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, ReplicationConfig, Server};

use asset_id::AssetId;
use asset_io::json::MeshData;

use editor_proto::components::{Edge3d, Face3d, FileExtension, FileType, Vertex3d};

use crate::{
    files::{FileWriter, ShapeTypeData},
    resources::{ContentEntityData, Project, ShapeManager},
};

// Writer
pub struct MeshWriter;

impl MeshWriter {
    fn world_to_data(
        &self,
        world: &mut World,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> MeshData {
        let content_entities = content_entities.keys().cloned().collect::<Vec<Entity>>();

        let mut system_state: SystemState<(
            Server,
            Res<ShapeManager>,
            Query<&Vertex3d>,
            Query<&Edge3d>,
            Query<&Face3d>,
            Query<&FileType>,
        )> = SystemState::new(world);
        let (server, shape_manager, vertex_q, edge_q, face_q, file_type_q) =
            system_state.get_mut(world);

        let mut output = MeshData::new();

        /////////////////////////////////////  id /////////////////
        let mut vertex_map: HashMap<Entity, usize> = HashMap::new();
        let mut edge_map: HashMap<Entity, usize> = HashMap::new();
        let mut face_list: Vec<Option<(u16, u16, u16, u16, u16, u16, u16)>> = Vec::new();

        info!(
            "writing in world_to_actions(), content_entities: `{:?}`",
            content_entities
        );

        let mut vertex_count: usize = 0;
        for entity in content_entities.iter() {
            if let Ok(vertex) = vertex_q.get(*entity) {
                // entity is a vertex
                check_for_mesh_file_type(&file_type_q, entity);
                vertex_map.insert(*entity, vertex_count);
                output.add_vertex(vertex.x(), vertex.y(), vertex.z());
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
                check_for_mesh_file_type(&file_type_q, entity);
                edge_map.insert(*entity, edge_count);
                let vertex_a_entity = edge.start.get(&server).unwrap();
                let vertex_b_entity = edge.end.get(&server).unwrap();
                let vertex_a_id = *vertex_map.get(&vertex_a_entity).unwrap();
                let vertex_b_id = *vertex_map.get(&vertex_b_entity).unwrap();
                output.add_edge(vertex_a_id as u16, vertex_b_id as u16);
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
                let Some(face_index) = shape_manager.get_face_index(entity) else {
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

                let face_info = (
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
            let Some((face_id, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c)) =
                face_info_opt
            else {
                panic!("face_list contains None");
            };
            output.add_face(
                face_id, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c,
            );
        }

        output
    }
}

fn check_for_mesh_file_type(file_type_q: &Query<&FileType>, entity: &Entity) {
    let Ok(file_type) = file_type_q.get(*entity) else {
        panic!("entity {:?} does not have a FileType component!", entity);
    };
    if *file_type.value != FileExtension::Mesh {
        panic!(
            "entity {:?} does not have a FileType component with value Mesh!",
            entity
        );
    }
}

impl FileWriter for MeshWriter {
    fn write(
        &self,
        world: &mut World,
        _project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
        asset_id: &AssetId,
    ) -> Box<[u8]> {
        let data = self.world_to_data(world, content_entities);
        data.write(asset_id)
    }

    fn write_new_default(&self, asset_id: &AssetId) -> Box<[u8]> {
        let mut data = MeshData::new();

        data.add_vertex(0, 0, 0);

        data.write(asset_id)
    }
}

// Reader
pub struct MeshReader;

impl MeshReader {
    fn data_to_world(
        world: &mut World,
        file_entity: &Entity,
        data: &MeshData,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut system_state: SystemState<(Commands, Server, ResMut<ShapeManager>)> =
            SystemState::new(world);
        let (mut commands, mut server, mut shape_manager) = system_state.get_mut(world);

        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut output = Vec::new();

        for vertex in data.get_vertices() {
            let (x, y, z) = vertex.deconstruct();
            let entity_id = commands
                .spawn_empty()
                .enable_replication(&mut server)
                .configure_replication(ReplicationConfig::Delegated)
                .insert(Vertex3d::new(x, y, z))
                .id();
            info!("spawning mesh vertex entity {:?}", entity_id);
            vertices.push(entity_id);
            output.push((entity_id, ShapeTypeData::Vertex));
        }
        for edge in data.get_edges() {
            let (vertex_a_index, vertex_b_index) = edge.deconstruct();

            let Some(vertex_a_entity) = vertices.get(vertex_a_index as usize) else {
                panic!(
                    "edge's vertex_a_index is `{:?}` and list of vertices is `{:?}`",
                    vertex_a_index, vertices
                );
            };
            let Some(vertex_b_entity) = vertices.get(vertex_b_index as usize) else {
                panic!(
                    "edge's vertex_b_index is `{:?}` and list of vertices is `{:?}`",
                    vertex_b_index, vertices
                );
            };

            let mut edge_component = Edge3d::new();
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
        for face in data.get_faces() {
            let (
                face_index,
                vertex_a_index,
                vertex_b_index,
                vertex_c_index,
                edge_a_index,
                edge_b_index,
                edge_c_index,
            ) = face.deconstruct();

            let vertex_a_entity = *vertices.get(vertex_a_index as usize).unwrap();
            let vertex_b_entity = *vertices.get(vertex_b_index as usize).unwrap();
            let vertex_c_entity = *vertices.get(vertex_c_index as usize).unwrap();

            let edge_a_entity = *edges.get(edge_a_index as usize).unwrap();
            let edge_b_entity = *edges.get(edge_b_index as usize).unwrap();
            let edge_c_entity = *edges.get(edge_c_index as usize).unwrap();

            let mut face_component = Face3d::new();
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
                "spawning mesh face entity `{:?}`, index is {:?}",
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

        let output = MeshReader::post_process_entities(&mut shape_manager, file_entity, output);

        system_state.apply(world);

        output
    }

    pub fn read(
        &self,
        world: &mut World,
        file_entity: &Entity,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {
        let Ok((meta, data)) = MeshData::read(bytes) else {
            panic!("Error reading .mesh file");
        };

        if meta.schema_version() != MeshData::CURRENT_SCHEMA_VERSION {
            panic!("Invalid schema version");
        }

        let result = Self::data_to_world(world, file_entity, &data);

        result
    }

    pub fn post_process_entities(
        shape_manager: &mut ShapeManager,
        file_entity: &Entity,
        shape_entities: Vec<(Entity, ShapeTypeData)>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut new_content_entities = HashMap::new();

        for (entity, shape_type_data) in shape_entities {
            new_content_entities
                .insert(entity, ContentEntityData::new_shape(shape_type_data.into()));

            match shape_type_data {
                ShapeTypeData::Vertex => {
                    shape_manager.on_create_mesh_vertex(entity);
                }
                ShapeTypeData::Edge(start, end) => {
                    shape_manager.on_create_mesh_edge(start, entity, end);
                }
                ShapeTypeData::Face(index, vert_a, vert_b, vert_c) => {
                    shape_manager.on_create_face(
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
