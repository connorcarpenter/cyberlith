use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, Res, ResMut, SystemState},
};
use logging::info;

use naia_bevy_server::{CommandsExt, ReplicationConfig, Server};

use asset_id::AssetId;
use asset_serde::json::{IconFileFrame, IconJson};

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
    // frame entity
    Vertex(Entity),
    // frame entity, start vertex, end vertex
    Edge(Entity, Entity, Entity),
    // face index, palette index, frame entity, vertex a, vertex b, vertex c
    Face(usize, u8, Entity, Entity, Entity, Entity),
}

struct IconFrameActionData {
    vertex_count: usize,
    edge_count: usize,
    vertex_map: HashMap<Entity, usize>,
    edge_map: HashMap<Entity, usize>,
    face_list: Vec<Option<(u16, u8, u16, u16, u16)>>,
    frame: IconFileFrame,
}

impl IconFrameActionData {
    fn new() -> Self {
        Self {
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
            let Some((face_id, color_id, vertex_a, vertex_b, vertex_c)) = face_info_opt else {
                panic!("face_list contains None");
            };
            self.frame
                .add_face(face_id, color_id, vertex_a, vertex_b, vertex_c);
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
    ) -> IconJson {
        let working_file_entries = project.working_file_entries();

        let mut output = IconJson::new();

        let mut palette_dependency_key_opt = None;
        let mut vertex_entities = Vec::new();
        let mut edge_entities = Vec::new();
        let mut face_entities = Vec::new();
        let mut frame_map: HashMap<Entity, IconFrameActionData> = HashMap::new();

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
                    frame_map.insert(*content_entity, IconFrameActionData::new());
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
            // let Ok(frame) = frame_q.get(frame_entity) else {
            //     panic!("");
            // };
            // let frame_index = frame.get_order() as u16;

            let frame_action_data = frame_map
                .get_mut(&frame_entity)
                .expect("should have initialized this already");
            frame_action_data.add_vertex(vertex_entity, vertex.x(), vertex.y());
        }

        // Write Edges
        for edge_entity in edge_entities {
            let Ok(edge) = edge_q.get(edge_entity) else {
                panic!("");
            };

            let frame_entity = edge.frame_entity.get(&server).unwrap();
            // let Ok(frame) = frame_q.get(frame_entity) else {
            //     panic!("");
            // };
            // let frame_index = frame.get_order() as u16;

            let frame_action_data = frame_map
                .get_mut(&frame_entity)
                .expect("should have initialized this already");

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
            // let Ok(frame) = frame_q.get(frame_entity) else {
            //     panic!("");
            // };
            // let frame_index = frame.get_order() as u16;

            let frame_action_data = frame_map
                .get_mut(&frame_entity)
                .expect("should have initialized this already");

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

        //

        let mut new_frame_list = Vec::new();

        for (frame_entity, frame_data) in frame_map.drain() {
            let Ok(frame_component) = frame_q.get(frame_entity) else {
                panic!("");
            };
            let frame_index = frame_component.get_order() as usize;
            // resize if necessary
            if frame_index >= new_frame_list.len() {
                new_frame_list.resize(frame_index + 1, None);
            }

            if new_frame_list[frame_index].is_some() {
                panic!(
                    "Duplicate frame order! (entity: {:?}, order index: {:?})",
                    frame_entity, frame_index
                );
            } else {
                info!(
                    "writing frame (entity: {:?}, order index {:?})",
                    frame_entity, frame_index
                );
            }
            new_frame_list[frame_index] = Some(frame_data.frame);
        }

        let mut frame_index = 0;
        for frame_opt in new_frame_list {
            let frame_data = frame_opt.expect("frame_opt should not be None");

            info!("adding IconAction::Frame({})", frame_index);

            output.add_frame(frame_data);

            frame_index += 1;
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
        let mut data = IconJson::new();
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
        data: &IconJson,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut system_state: SystemState<(Commands, Server, ResMut<IconManager>)> =
            SystemState::new(world);
        let (mut commands, mut server, mut icon_manager) = system_state.get_mut(world);

        let mut output = Vec::new();
        let mut frame_index = 0;

        // Palette Dependency
        if let Some(asset_id) = data.get_palette_asset_id() {
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
        }

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
                output.push((entity_id, ContentEntityTypeData::Vertex(frame_entity)));
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
                    ContentEntityTypeData::Edge(frame_entity, *vertex_a_entity, *vertex_b_entity),
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
        let Ok((meta, data)) = IconJson::read(bytes) else {
            panic!("Error reading .icon file");
        };

        if meta.schema_version() != IconJson::CURRENT_SCHEMA_VERSION {
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
                ContentEntityTypeData::Vertex(frame_entity) => {
                    icon_manager.on_create_vertex(frame_entity, entity);

                    new_content_entities
                        .insert(entity, ContentEntityData::new_icon_shape(ShapeType::Vertex));
                }
                ContentEntityTypeData::Edge(frame_entity, start, end) => {
                    icon_manager.on_create_edge(frame_entity, start, entity, end);

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
