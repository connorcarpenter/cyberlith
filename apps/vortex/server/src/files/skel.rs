use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, Res, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{
    BitReader, CommandsExt, FileBitWriter, ReplicationConfig, Serde, SerdeErr, Server,
    UnsignedInteger, UnsignedVariableInteger,
};

use vortex_proto::{
    components::{
        Edge3d, EdgeAngle, FileExtension, FileType, ShapeName, Vertex3d, VertexRoot, VertexSerdeInt,
    },
};

use crate::{
    files::{
        file_io::ShapeType, FileReadOutput, FileReader, FileWriter, SkelFileWaitlist,
        SkelWaitlistInsert,
    },
    resources::{ContentEntityData, Project, ShapeManager},
};

// Actions
#[derive(Debug)]
enum SkelAction {
    //////// x,   y,   z, Option<parent_id, angle>, vertex_name, edge_name //
    Vertex(
        i16,
        i16,
        i16,
        Option<(u16, UnsignedInteger<6>)>,
        Option<String>,
        Option<String>,
    ),
}

// Writer
pub struct SkelWriter;

impl SkelWriter {
    fn new_default_actions(&self) -> Vec<SkelAction> {
        let mut output = Vec::new();

        output.push(SkelAction::Vertex(0, 0, 0, None, None, None));

        output
    }

    fn world_to_actions(
        &self,
        world: &mut World,
        content_entities_opt: &Option<HashMap<Entity, ContentEntityData>>,
    ) -> Vec<SkelAction> {
        let content_entities = content_entities_opt
            .as_ref()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<Entity>>();

        let mut system_state: SystemState<(
            Res<ShapeManager>,
            Query<&Vertex3d>,
            Query<&FileType>,
            Query<&ShapeName>,
            Query<&EdgeAngle>,
        )> = SystemState::new(world);
        let (shape_manager, vertex_q, file_type_q, shape_name_q, edge_angle_q) =
            system_state.get_mut(world);

        let mut output = Vec::new();

        ///////////////////////////////  id,   x,   y,   z, Option<parent_entity, angle>, vertex_name, edge_name ///////////////////
        let mut map: HashMap<
            Entity,
            (
                usize,
                i16,
                i16,
                i16,
                Option<(Entity, UnsignedInteger<6>)>,
                Option<String>,
                Option<String>,
            ),
        > = HashMap::new();
        let mut vertices: Vec<Entity> = Vec::new();

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
                        Some((*shape_name.value).clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            let edge_name_opt: Option<String> = {
                if let Some((_, edge_entity)) = parent_and_edge_entity_opt {
                    if let Ok(shape_name) = shape_name_q.get(edge_entity) {
                        if shape_name.value.len() > 0 {
                            Some((*shape_name.value).clone())
                        } else {
                            None
                        }
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
                        panic!("edge_entity {:?} does not have an EdgeAngle component!", edge_entity);
                    };
                    (parent_entity, *edge_angle.value)
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
                    edge_name_opt,
                ),
            );
            vertices.push(*entity);
        }

        for entity in vertices.iter() {
            let (_, x, y, z, parent_entity_opt, vertex_name_opt, edge_name_opt) =
                map.get(entity).unwrap();
            let parent_id = parent_entity_opt.map(|(parent_entity, angle)| {
                let (parent_id, _, _, _, _, _, _) = map.get(&parent_entity).unwrap();
                (*parent_id as u16, angle)
            });
            let vertex_info = SkelAction::Vertex(
                *x,
                *y,
                *z,
                parent_id,
                vertex_name_opt.clone(),
                edge_name_opt.clone(),
            );
            output.push(vertex_info);
        }

        output
    }

    fn write_from_actions(&self, actions: Vec<SkelAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                SkelAction::Vertex(x, y, z, parent_id_opt, vertex_name_opt, edge_name_opt) => {
                    info!("writing vertex (x: {:?}, y: {:?}, z: {:?}, parent_id_opt: {:?}, vertex_name_opt: {:?}, edge_name_opt: {:?})", x, y, z, parent_id_opt, vertex_name_opt, edge_name_opt);

                    // continue bit
                    true.ser(&mut bit_writer);

                    // encode X, Y, Z
                    VertexSerdeInt::from(x).ser(&mut bit_writer);
                    VertexSerdeInt::from(y).ser(&mut bit_writer);
                    VertexSerdeInt::from(z).ser(&mut bit_writer);

                    // Parent Id
                    let parent_id = {
                        if let Some((parent_id, _)) = parent_id_opt {
                            parent_id + 1
                        } else {
                            0
                        }
                    };
                    UnsignedVariableInteger::<6>::from(parent_id).ser(&mut bit_writer);

                    // Angle
                    if let Some((_, angle)) = parent_id_opt {
                        angle.ser(&mut bit_writer);
                    }

                    // Names
                    vertex_name_opt.ser(&mut bit_writer);
                    edge_name_opt.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        false.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

impl FileWriter for SkelWriter {
    fn write(
        &self,
        world: &mut World,
        _project: &Project,
        content_entities_opt: &Option<HashMap<Entity, ContentEntityData>>,
    ) -> Box<[u8]> {
        let actions = self.world_to_actions(world, content_entities_opt);
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
            let parent_and_angle_opt = if let Some(parent_id) = parent_id_opt {
                let angle = UnsignedInteger::<6>::de(bit_reader)?;
                Some((parent_id, angle))
            } else {
                None
            };
            let vertex_name_opt = Option::<String>::de(bit_reader)?;
            let edge_name_opt = Option::<String>::de(bit_reader)?;

            output.push(SkelAction::Vertex(
                x,
                y,
                z,
                parent_and_angle_opt,
                vertex_name_opt,
                edge_name_opt,
            ));
        }
        Ok(output)
    }

    fn actions_to_world(
        commands: &mut Commands,
        server: &mut Server,
        actions: Vec<SkelAction>,
    ) -> Result<FileReadOutput, SerdeErr> {
        let mut output = Vec::new();

        let mut entities: Vec<(
            Entity,
            i16,
            i16,
            i16,
            Option<(u16, UnsignedInteger<6>)>,
            Option<String>,
            Option<String>,
        )> = Vec::new();

        for action in actions {
            match action {
                SkelAction::Vertex(x, y, z, parent_id_opt, vertex_name_opt, edge_name_opt) => {
                    let entity_id = commands.spawn_empty().enable_replication(server).id();
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
                            parent_id_opt,
                            vertex_name_opt,
                            edge_name_opt,
                        ));
                    } else {
                        // root node should always be at 0,0,0 ... you can refactor these files later
                        entities.push((
                            entity_id,
                            0,
                            0,
                            0,
                            parent_id_opt,
                            vertex_name_opt,
                            edge_name_opt,
                        ));
                    }
                }
            }
        }

        for (entity, x, y, z, parent_id_opt, vertex_name_opt, edge_name_opt) in entities.iter() {
            commands.entity(*entity).insert(Vertex3d::new(*x, *y, *z));

            if let Some(vertex_name) = vertex_name_opt {
                commands
                    .entity(*entity)
                    .insert(ShapeName::new(vertex_name.clone()));
            }

            if let Some((parent_id, edge_angle)) = parent_id_opt {
                let Some((parent_entity, _, _, _, _, _, _)) = entities.get(*parent_id as usize) else {
                    panic!("parent_id {:?} not found", parent_id);
                };

                let mut edge_component = Edge3d::new();
                edge_component.start.set(server, parent_entity);
                edge_component.end.set(server, entity);
                let edge_entity = commands
                    .spawn_empty()
                    .enable_replication(server)
                    // setting to Delegated to match client-created edges
                    .configure_replication(ReplicationConfig::Delegated)
                    .insert(edge_component)
                    .insert(EdgeAngle::new_complete(*edge_angle))
                    .id();

                if let Some(edge_name) = edge_name_opt {
                    commands
                        .entity(edge_entity)
                        .insert(ShapeName::new(edge_name.clone()));
                }

                output.push((*entity, Some((edge_entity, *parent_entity))));
            } else {
                commands.entity(*entity).insert(VertexRoot);
                output.push((*entity, None));
            }
        }

        Ok(FileReadOutput::Skel(output))
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
