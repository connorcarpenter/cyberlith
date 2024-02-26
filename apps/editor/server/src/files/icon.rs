use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, Res, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, ReplicationConfig, Server};

use asset_id::AssetId;
use asset_io::json::{IconFile, IconFileFrame};

use editor_proto::{
    components::{FileExtension, IconEdge, IconFace, IconFrame, IconVertex, PaletteColor},
    resources::FileKey,
};

use crate::{
    files::{add_file_dependency, FileWriter, ShapeType},
    resources::{ContentEntityData, IconManager, Project},
};

#[derive(Clone)]
enum ContentEntityTypeData {
    Dependency(FileKey),
    Frame,
    Vertex,
    Edge(Entity, Entity),
    // face index, palette index, frame entity, vertex a, vertex b, vertex c
    Face(usize, u8, Entity, Entity, Entity, Entity),
}

struct IconFrameActionData {
    frame_entity: Entity,
    vertex_count: usize,
    edge_count: usize,
    vertex_map: HashMap<Entity, usize>,
    edge_map: HashMap<Entity, usize>,
    face_list: Vec<Option<(u16, u8, u16, u16, u16)>>,
    frame: IconFileFrame,
}

impl IconFrameActionData {
    fn new(frame_entity: Entity) -> Self {
        Self {
            frame_entity,
            vertex_count: 0,
            edge_count: 0,
            vertex_map: HashMap::new(),
            edge_map: HashMap::new(),
            face_list: Vec::new(),
            frame: IconFileFrame::new(),
        }
    }

    fn add_vertex(&mut self, vertex_entity: Entity, x: i16, y: i16) {
        // entity is a vertex
        //info!("add_vertex - {}: `{:?}`", self.vertex_count, vertex_entity);
        self.vertex_map.insert(vertex_entity, self.vertex_count);
        self.frame.add_vertex(x, y);
        self.vertex_count += 1;
    }

    fn add_edge(&mut self, edge_entity: Entity, vertex_a_entity: Entity, vertex_b_entity: Entity) {
        // entity is an edge
        //info!("add_edge - {}: `{:?}` .. vertex a: `{:?}` -> vertex_b: `{:?}`", self.edge_count, edge_entity,vertex_a_entity, vertex_b_entity);
        self.edge_map.insert(edge_entity, self.edge_count);

        let vertex_a_id = *self.vertex_map.get(&vertex_a_entity).unwrap();
        let vertex_b_id = *self.vertex_map.get(&vertex_b_entity).unwrap();
        self.frame.add_edge(vertex_a_id as u16, vertex_b_id as u16);
        self.edge_count += 1;
    }

    fn add_face(
        &mut self,
        face_index: u16,
        palette_color_index: u8,
        vertex_a_entity: Entity,
        vertex_b_entity: Entity,
        vertex_c_entity: Entity,
    ) {
        let vertex_a_id = *self.vertex_map.get(&vertex_a_entity).unwrap();
        let vertex_b_id = *self.vertex_map.get(&vertex_b_entity).unwrap();
        let vertex_c_id = *self.vertex_map.get(&vertex_c_entity).unwrap();

        let face_info = (
            face_index,
            palette_color_index,
            vertex_a_id as u16,
            vertex_b_id as u16,
            vertex_c_id as u16,
        );
        if face_index as usize >= self.face_list.len() {
            self.face_list.resize((face_index + 1) as usize, None);
        }
        self.face_list[face_index as usize] = Some(face_info);
    }

    fn complete_faces(&mut self) {
        let face_list = std::mem::take(&mut self.face_list);

        info!("writing face list: {:?}", face_list);

        for face_info_opt in face_list {
            let Some((face_id, color_id, vertex_a, vertex_b, vertex_c)) =
                face_info_opt
            else {
                panic!("face_list contains None");
            };
            self.frame.add_face(
                face_id, color_id, vertex_a, vertex_b, vertex_c,
            );
        }
    }

    pub(crate) fn validate_frame_entity(&self, frame_entity: &Entity) {
        if *frame_entity != self.frame_entity {
            panic!(
                "frame_entity does not match. expected: `{:?}`, got: `{:?}`",
                self.frame_entity, frame_entity
            );
        }

    }
}

// Writer
pub struct IconWriter;

impl IconWriter {
    fn world_to_data(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> IconFile {
        let working_file_entries = project.working_file_entries();

        let mut output = IconFile::new();

        let mut palette_dependency_key_opt = None;
        let mut vertex_entities = Vec::new();
        let mut edge_entities = Vec::new();
        let mut face_entities = Vec::new();
        let mut frame_map: HashMap<u16, IconFrameActionData> = HashMap::new();
        let mut frame_index = 0;

        for (content_entity, content_data) in content_entities {
            match content_data {
                ContentEntityData::Dependency(dependency_key) => {
                    let dependency_value = working_file_entries.get(dependency_key).unwrap();
                    let dependency_file_ext = dependency_value.extension().unwrap();
                    match dependency_file_ext {
                        FileExtension::Palette => {
                            palette_dependency_key_opt = Some(dependency_key);
                        }
                        _ => {
                            panic!("icon file should depend on a single .palette file");
                        }
                    }
                }
                ContentEntityData::IconShape(ShapeType::Vertex) => {
                    vertex_entities.push(*content_entity);
                }
                ContentEntityData::IconShape(ShapeType::Edge) => {
                    edge_entities.push(*content_entity);
                }
                ContentEntityData::IconShape(ShapeType::Face) => {
                    panic!("invalid type");
                }
                ContentEntityData::IconFace(_) => {
                    face_entities.push(*content_entity);
                }
                ContentEntityData::Frame => {
                    // here we're just filling up frame_map, to be written later
                    frame_map.insert(frame_index, IconFrameActionData::new(*content_entity));
                    frame_index += 1;
                }
                _ => {
                    panic!(
                        "icon should not have this content entity type: {:?}",
                        content_data
                    );
                }
            }
        }

        // Write Palette Dependency
        if let Some(dependency_key) = palette_dependency_key_opt {
            info!("writing palette dependency: {}", dependency_key.full_path());
            let asset_id = project.asset_id(&dependency_key).unwrap();
            output.set_palette_asset_id(&asset_id);
        }

        let mut system_state: SystemState<(
            Server,
            Res<IconManager>,
            Query<&PaletteColor>,
            Query<&IconVertex>,
            Query<&IconEdge>,
            Query<&IconFace>,
            Query<&IconFrame>,
        )> = SystemState::new(world);
        let (server, icon_manager, palette_color_q, vertex_q, edge_q, face_q, frame_q) =
            system_state.get_mut(world);

        // Write Vertices
        for vertex_entity in vertex_entities {
            let Ok(vertex) = vertex_q.get(vertex_entity) else {
                panic!("");
            };

            let frame_entity = vertex.frame_entity.get(&server).unwrap();
            let Ok(frame) = frame_q.get(frame_entity) else {
                panic!("");
            };
            let frame_index = frame.get_order() as u16;

            let frame_action_data = frame_map
                .get_mut(&frame_index)
                .expect("should have initialized this already");
            frame_action_data.validate_frame_entity(&frame_entity);
            frame_action_data.add_vertex(vertex_entity, vertex.x(), vertex.y());
        }

        // Write Edges
        for edge_entity in edge_entities {
            let Ok(edge) = edge_q.get(edge_entity) else {
                panic!("");
            };

            let frame_entity = edge.frame_entity.get(&server).unwrap();
            let Ok(frame) = frame_q.get(frame_entity) else {
                panic!("");
            };
            let frame_index = frame.get_order() as u16;

            let frame_action_data = frame_map
                .get_mut(&frame_index)
                .expect("should have initialized this already");
            frame_action_data.validate_frame_entity(&frame_entity);

            let vertex_a_entity = edge.start.get(&server).unwrap();
            let vertex_b_entity = edge.end.get(&server).unwrap();

            frame_action_data.add_edge(edge_entity, vertex_a_entity, vertex_b_entity);
        }

        // Write Faces
        for face_entity in face_entities {
            let Ok(face) = face_q.get(face_entity) else {
                panic!("");
            };

            let frame_entity = face.frame_entity.get(&server).unwrap();
            let palette_color_entity = face.palette_color_entity.get(&server).unwrap();
            let Ok(frame) = frame_q.get(frame_entity) else {
                panic!("");
            };
            let frame_index = frame.get_order() as u16;

            let frame_action_data = frame_map
                .get_mut(&frame_index)
                .expect("should have initialized this already");
            frame_action_data.validate_frame_entity(&frame_entity);

            let Some(face_index) = icon_manager.get_face_index(&face_entity) else {
                panic!("face entity {:?} does not have an index!", face_entity);
            };
            let palette_color = palette_color_q.get(palette_color_entity).unwrap();
            let palette_color_index = *palette_color.index;

            let vertex_a_entity = face.vertex_a.get(&server).unwrap();
            let vertex_b_entity = face.vertex_b.get(&server).unwrap();
            let vertex_c_entity = face.vertex_c.get(&server).unwrap();

            frame_action_data.add_face(
                face_index as u16,
                palette_color_index,
                vertex_a_entity,
                vertex_b_entity,
                vertex_c_entity,
            );
        }

        for (_, value) in frame_map.iter_mut() {
            value.complete_faces();
        }

        // Write Frames
        let mut frame_index = 0;

        while frame_map.contains_key(&frame_index) {
            info!("adding IconAction::Frame({})", frame_index);

            let frame_action_data = frame_map.remove(&frame_index).unwrap();

            {
                // check that frame index is correct
                let Ok(frame) = frame_q.get(frame_action_data.frame_entity) else {
                    panic!("");
                };
                let data_frame_index = frame.get_order() as u16;
                if data_frame_index != frame_index {
                    panic!(
                        "frame index does not match. expected: `{:?}`, got: `{:?}`",
                        frame_index, data_frame_index
                    );
                }
            }

            output.add_frame(frame_action_data.frame);
            frame_index += 1;
        }

        if frame_map.len() > 0 {
            panic!("frame_map should be empty!");
        }

        output
    }
}

impl FileWriter for IconWriter {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
        asset_id: &AssetId,
    ) -> Box<[u8]> {
        let data = self.world_to_data(world, project, content_entities);
        data.write(asset_id)
    }

    fn write_new_default(&self, asset_id: &AssetId) -> Box<[u8]> {
        let mut data = IconFile::new();
        let mut frame = IconFileFrame::new();
        frame.add_vertex(0, 0);
        data.add_frame(frame);

        data.write(asset_id)
    }
}

// Reader
pub struct IconReader;

impl IconReader {
    fn data_to_world(
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        data: &IconFile,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut system_state: SystemState<(Commands, Server, ResMut<IconManager>)> =
            SystemState::new(world);
        let (mut commands, mut server, mut icon_manager) = system_state.get_mut(world);

        let mut output = Vec::new();
        let mut frame_index = 0;

        // Palette Dependency
        let asset_id = data.get_palette_asset_id();
        let dependency_key = project.file_key_from_asset_id(&asset_id).unwrap();
        let (new_entity, _) = add_file_dependency(
            project,
            file_key,
            file_entity,
            &mut commands,
            &mut server,
            FileExtension::Palette,
            &dependency_key,
        );
        output.push((
            new_entity,
            ContentEntityTypeData::Dependency(dependency_key),
        ));

        // Frames
        for frame in data.get_frames() {
            // make frame entity
            let mut component = IconFrame::new(frame_index);
            component.file_entity.set(&server, file_entity);
            let frame_entity = commands
                .spawn_empty()
                .enable_replication(&mut server)
                .configure_replication(ReplicationConfig::Delegated)
                .insert(component)
                .id();
            info!(
                "spawning icon frame entity. index: {:?}, entity: `{:?}`",
                frame_index, frame_entity
            );

            output.push((frame_entity, ContentEntityTypeData::Frame));

            icon_manager.on_create_frame(&file_entity, &frame_entity, frame_index as usize, None);

            frame_index += 1;

            // make frame contents
            let mut vertices = Vec::new();
            let mut edges = Vec::new();

            for vertex in frame.get_vertices() {
                let mut component = IconVertex::new(vertex.x(), vertex.y());
                component.frame_entity.set(&server, &frame_entity);
                let entity_id = commands
                    .spawn_empty()
                    .enable_replication(&mut server)
                    .configure_replication(ReplicationConfig::Delegated)
                    .insert(component)
                    .id();
                info!("spawning icon vertex entity {:?}", entity_id);
                vertices.push(entity_id);
                output.push((entity_id, ContentEntityTypeData::Vertex));
            }
            for edge in frame.get_edges() {
                let Some(vertex_a_entity) = vertices.get(edge.vertex_a() as usize) else {
                    panic!(
                        "edge's vertex_a_index is `{:?}` and list of vertices is `{:?}`",
                        edge.vertex_a(),
                        vertices
                    );
                };
                let Some(vertex_b_entity) = vertices.get(edge.vertex_b() as usize) else {
                    panic!(
                        "edge's vertex_b_index is `{:?}` and list of vertices is `{:?}`",
                        edge.vertex_b(),
                        vertices
                    );
                };

                let mut edge_component = IconEdge::new();
                edge_component.frame_entity.set(&server, &frame_entity);
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
                    ContentEntityTypeData::Edge(*vertex_a_entity, *vertex_b_entity),
                ));
            }
            for face in frame.get_faces() {
                let vertex_a_entity = *vertices.get(face.vertex_a() as usize).unwrap();
                let vertex_b_entity = *vertices.get(face.vertex_b() as usize).unwrap();
                let vertex_c_entity = *vertices.get(face.vertex_c() as usize).unwrap();

                let mut face_component = IconFace::new();
                face_component.frame_entity.set(&server, &frame_entity);
                face_component.vertex_a.set(&server, &vertex_a_entity);
                face_component.vertex_b.set(&server, &vertex_b_entity);
                face_component.vertex_c.set(&server, &vertex_c_entity);

                let entity_id = commands
                    .spawn_empty()
                    .enable_replication(&mut server)
                    // setting to Delegated to match client-created faces
                    .configure_replication(ReplicationConfig::Delegated)
                    .insert(face_component)
                    .id();
                info!(
                    "spawning icon face entity `{:?}`, index is {:?}",
                    entity_id,
                    face.face_id(),
                );
                output.push((
                    entity_id,
                    ContentEntityTypeData::Face(
                        face.face_id() as usize,
                        face.color_id(),
                        frame_entity,
                        vertex_a_entity,
                        vertex_b_entity,
                        vertex_c_entity,
                    ),
                ));
            }
        }

        let output = IconReader::post_process_entities(&mut icon_manager, output);

        system_state.apply(world);

        output
    }

    pub fn read(
        &self,
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {
        let Ok((meta, data)) = IconFile::read(bytes) else {
            panic!("Error reading .icon file");
        };

        if meta.schema_version() != IconFile::CURRENT_SCHEMA_VERSION {
            panic!("Invalid schema version");
        }

        let result = Self::data_to_world(world, project, file_key, file_entity, &data);

        result
    }

    // TODO: move this into the main read functions
    fn post_process_entities(
        icon_manager: &mut IconManager,
        shape_entities: Vec<(Entity, ContentEntityTypeData)>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut new_content_entities = HashMap::new();

        for (entity, content_entity_type_data) in shape_entities {
            match content_entity_type_data {
                ContentEntityTypeData::Dependency(file_key) => {
                    new_content_entities
                        .insert(entity, ContentEntityData::new_dependency(file_key));
                }
                ContentEntityTypeData::Frame => {
                    new_content_entities.insert(entity, ContentEntityData::new_frame());
                }
                ContentEntityTypeData::Vertex => {
                    icon_manager.on_create_vertex(entity);

                    new_content_entities
                        .insert(entity, ContentEntityData::new_icon_shape(ShapeType::Vertex));
                }
                ContentEntityTypeData::Edge(start, end) => {
                    icon_manager.on_create_edge(start, entity, end);

                    new_content_entities
                        .insert(entity, ContentEntityData::new_icon_shape(ShapeType::Edge));
                }
                ContentEntityTypeData::Face(
                    index,
                    palette_index,
                    frame_entity,
                    vert_a,
                    vert_b,
                    vert_c,
                ) => {
                    icon_manager.on_create_face(
                        &frame_entity,
                        Some(index),
                        entity,
                        vert_a,
                        vert_b,
                        vert_c,
                    );

                    new_content_entities.insert(
                        entity,
                        ContentEntityData::new_icon_face(Some(palette_index)),
                    );
                }
            }
        }

        new_content_entities
    }
}
