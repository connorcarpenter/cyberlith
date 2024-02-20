use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, Res, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, ReplicationConfig, Server};

use asset_io::json::SkelFile;
use asset_io::AssetId;

use editor_proto::components::{
    Edge3d, EdgeAngle, FileExtension, FileType, SerdeRotation, ShapeName, Vertex3d, VertexRoot,
};

use crate::{
    files::{
        convert_from_rotation, convert_into_rotation, file_io::ShapeType, FileWriter,
        SkelFileWaitlist, SkelWaitlistInsert,
    },
    resources::{ContentEntityData, Project, ShapeManager},
};

// Writer
pub struct SkelWriter;

impl SkelWriter {
    fn new_default_data(&self) -> SkelFile {
        let mut output = SkelFile::new();

        output.add_vertex(0, 0, 0, None, None);

        output
    }

    fn world_to_data(
        &self,
        world: &mut World,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> SkelFile {
        let content_entities = content_entities.keys().cloned().collect::<Vec<Entity>>();

        let mut system_state: SystemState<(
            Res<ShapeManager>,
            Query<&Vertex3d>,
            Query<&FileType>,
            Query<&ShapeName>,
            Query<&EdgeAngle>,
        )> = SystemState::new(world);
        let (shape_manager, vertex_q, file_type_q, shape_name_q, edge_angle_q) =
            system_state.get_mut(world);

        let mut output = SkelFile::new();

        ///////////////////////////////  id,   x,   y,   z, Option<parent_entity, angle>, vertex_name ///////////////////
        let mut map: HashMap<
            Entity,
            (
                usize,
                i16,
                i16,
                i16,
                Option<(Entity, SerdeRotation)>,
                Option<String>,
            ),
        > = HashMap::new();
        let mut vertices: Vec<Entity> = Vec::new();
        let mut vertex_names: HashSet<String> = HashSet::new();

        for entity in content_entities.iter() {
            let Ok(file_type) = file_type_q.get(*entity) else {
                panic!("entity {:?} does not have a FileType component!", entity);
            };
            if *file_type.value != FileExtension::Skel {
                panic!(
                    "entity {:?} does not have a FileType component with value Skel!",
                    entity
                );
            }
            let Ok(vertex) = vertex_q.get(*entity) else {
                continue;
            };

            let parent_and_edge_entity_opt: Option<(Entity, Entity)> =
                shape_manager.get_vertex_parent_and_edge(entity);

            let vertex_name_opt: Option<String> = {
                if let Ok(shape_name) = shape_name_q.get(*entity) {
                    if shape_name.value.len() > 0 {
                        let value = (*shape_name.value).clone();
                        if vertex_names.contains(&value) {
                            panic!("vertex name {:?} already exists in file!", value);
                        } else {
                            vertex_names.insert(value.clone());
                        }
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            let parent_entity_opt =
                parent_and_edge_entity_opt.map(|(parent_entity, edge_entity)| {
                    let Ok(edge_angle) = edge_angle_q.get(edge_entity) else {
                        panic!(
                            "edge_entity {:?} does not have an EdgeAngle component!",
                            edge_entity
                        );
                    };
                    (parent_entity, edge_angle.get_serde())
                });

            let id = vertices.len();
            map.insert(
                *entity,
                (
                    id,
                    vertex.x(),
                    vertex.y(),
                    vertex.z(),
                    parent_entity_opt,
                    vertex_name_opt,
                ),
            );
            vertices.push(*entity);
        }

        for entity in vertices.iter() {
            let (_, x, y, z, parent_entity_opt, vertex_name_opt) = map.get(entity).unwrap();
            let parent_id = parent_entity_opt.map(|(parent_entity, angle)| {
                let (parent_id, _, _, _, _, _) = map.get(&parent_entity).unwrap();
                (*parent_id as u16, angle)
            });
            output.add_vertex(
                *x,
                *y,
                *z,
                parent_id.map(|(id, rot)| (id, convert_into_rotation(rot))),
                vertex_name_opt.clone(),
            );
        }

        output
    }
}

impl FileWriter for SkelWriter {
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
        let data = self.new_default_data();
        data.write(asset_id)
    }
}

// Reader
pub struct SkelReader;

impl SkelReader {
    fn data_to_world(world: &mut World, data: &SkelFile) -> HashMap<Entity, ContentEntityData> {
        let mut system_state: SystemState<(Commands, Server, ResMut<ShapeManager>)> =
            SystemState::new(world);
        let (mut commands, mut server, mut shape_manager) = system_state.get_mut(world);

        let mut output = Vec::new();

        let mut entities: Vec<(
            Entity,
            i16,
            i16,
            i16,
            Option<(u16, SerdeRotation)>,
            Option<String>,
        )> = Vec::new();

        for vertex in data.get_vertices() {
            let (x, y, z, parent_id_opt, vertex_name_opt) = vertex.deconstruct();

            let entity_id = commands.spawn_empty().enable_replication(&mut server).id();
            info!(
                "spawning vertex (id {:?}, entity: {:?}, parent_id_opt: {:?})",
                entities.len(),
                entity_id,
                parent_id_opt
            );
            commands
                .entity(entity_id)
                .configure_replication(ReplicationConfig::Delegated);
            if parent_id_opt.is_some() {
                entities.push((
                    entity_id,
                    x,
                    y,
                    z,
                    parent_id_opt.map(|(id, rot)| (id, convert_from_rotation(rot))),
                    vertex_name_opt,
                ));
            } else {
                // root node should always be at 0,0,0 ... you can refactor these files later
                entities.push((
                    entity_id,
                    0,
                    0,
                    0,
                    parent_id_opt.map(|(id, rot)| (id, convert_from_rotation(rot))),
                    vertex_name_opt,
                ));
            }
        }

        for (entity, x, y, z, parent_id_opt, vertex_name_opt) in entities.iter() {
            commands.entity(*entity).insert(Vertex3d::new(*x, *y, *z));

            if let Some(vertex_name) = vertex_name_opt {
                commands
                    .entity(*entity)
                    .insert(ShapeName::new(vertex_name.clone()));
            }

            if let Some((parent_id, edge_angle)) = parent_id_opt {
                let Some((parent_entity, _, _, _, _, _)) = entities.get(*parent_id as usize) else {
                    panic!("parent_id {:?} not found", parent_id);
                };

                let mut edge_component = Edge3d::new();
                edge_component.start.set(&server, parent_entity);
                edge_component.end.set(&server, entity);
                let edge_entity = commands
                    .spawn_empty()
                    .enable_replication(&mut server)
                    // setting to Delegated to match client-created edges
                    .configure_replication(ReplicationConfig::Delegated)
                    .insert(edge_component)
                    .insert(EdgeAngle::new_complete(*edge_angle))
                    .id();

                output.push((*entity, Some((edge_entity, *parent_entity))));
            } else {
                commands.entity(*entity).insert(VertexRoot);
                output.push((*entity, None));
            }
        }

        let output = SkelReader::post_process_entities(&mut shape_manager, output);

        system_state.apply(world);

        output
    }

    pub fn post_process_entities(
        shape_manager: &mut ShapeManager,
        vertex_and_edge_entities: Vec<(Entity, Option<(Entity, Entity)>)>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut new_content_entities = HashMap::new();

        let mut skel_file_waitlist = SkelFileWaitlist::default();

        for (vertex_entity, edge_opt) in vertex_and_edge_entities {
            new_content_entities.insert(
                vertex_entity,
                ContentEntityData::new_shape(ShapeType::Vertex),
            );

            if let Some((edge_entity, parent_entity)) = edge_opt {
                skel_file_waitlist
                    .process_insert(shape_manager, SkelWaitlistInsert::Vertex(vertex_entity));
                skel_file_waitlist.process_insert(
                    shape_manager,
                    SkelWaitlistInsert::Edge(parent_entity, edge_entity, vertex_entity),
                );
                new_content_entities
                    .insert(edge_entity, ContentEntityData::new_shape(ShapeType::Edge));
            } else {
                skel_file_waitlist
                    .process_insert(shape_manager, SkelWaitlistInsert::VertexRoot(vertex_entity));
            }
        }

        new_content_entities
    }

    pub fn read(&self, world: &mut World, bytes: &Box<[u8]>) -> HashMap<Entity, ContentEntityData> {
        let Ok((meta, data)) = SkelFile::read(bytes) else {
            panic!("Error reading .skel file");
        };

        if meta.schema_version() != SkelFile::CURRENT_SCHEMA_VERSION {
            panic!("Invalid schema version");
        }

        let result = Self::data_to_world(world, &data);

        result
    }
}
