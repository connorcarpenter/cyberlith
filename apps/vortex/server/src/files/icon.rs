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

use vortex_proto::{resources::FileKey, components::{FileExtension, IconBackgroundColor, IconEdge, IconFace, IconFrame, IconVertex, PaletteColor, VertexSerdeInt}};

use crate::{
    files::{add_file_dependency,FileWriter, ShapeType},
    resources::{ContentEntityData, IconManager, Project},
};

#[derive(Clone)]
enum ContentEntityTypeData {
    Dependency(FileKey),
    BackgroundColor(Option<u8>),
    Frame,
    Vertex,
    Edge(Entity, Entity),
    Face(usize, Entity, Entity, Entity),
}

#[derive(Debug, Clone)]
enum IconFrameAction {
    //////// x, y//
    Vertex(i16, i16),
    //// vertex id1, vertex id2 //
    Edge(u16, u16),
    //// order_index, id1, id2, id3 // (vertex ids) // id4, id5, id6 (edge ids) // TODO: remove order_index?
    Face(u16, u16, u16, u16, u16, u16, u16),
}

#[derive(Serde, Clone, PartialEq)]
enum IconFrameActionType {
    None,
    Vertex,
    Edge,
    Face,
}

// Actions
#[derive(Debug, Clone)]
enum IconAction {
    // path, file_name
    PaletteFile(String, String),
    // palette color index
    BackgroundColor(u8),
    // frame
    Frame(Vec<IconFrameAction>),
}

#[derive(Serde, Clone, PartialEq)]
enum IconActionType {
    None,
    PaletteFile,
    BackgroundColor,
    Frame,
}

struct IconFrameActionData {
    vertex_count: usize,
    edge_count: usize,
    vertex_map: HashMap<Entity, usize>,
    edge_map: HashMap<Entity, usize>,
    face_list: Vec<Option<IconFrameAction>>,
    frame_actions: Vec<IconFrameAction>,
}

impl IconFrameActionData {
    fn new() -> Self {
        Self {
            vertex_count: 0,
            edge_count: 0,
            vertex_map: HashMap::new(),
            edge_map: HashMap::new(),
            face_list: Vec::new(),
            frame_actions: Vec::new(),
        }
    }

    fn add_vertex(&mut self, vertex_entity: Entity, x: i16, y: i16) {
        // entity is a vertex
        self.vertex_map.insert(vertex_entity, self.vertex_count);
        let vertex_info = IconFrameAction::Vertex(x, y);
        self.frame_actions.push(vertex_info);
        self.vertex_count += 1;
    }

    fn add_edge(&mut self, edge_entity: Entity, vertex_a_entity: Entity, vertex_b_entity: Entity) {
        // entity is an edge
        self.edge_map.insert(edge_entity, self.edge_count);

        let vertex_a_id = *self.vertex_map.get(&vertex_a_entity).unwrap();
        let vertex_b_id = *self.vertex_map.get(&vertex_b_entity).unwrap();
        let edge_info = IconFrameAction::Edge(vertex_a_id as u16, vertex_b_id as u16);
        self.frame_actions.push(edge_info);
        self.edge_count += 1;
    }

    fn add_face(
        &mut self,
        face_index: u16,
        vertex_a_entity: Entity,
        vertex_b_entity: Entity,
        vertex_c_entity: Entity,
        edge_a_entity: Entity,
        edge_b_entity: Entity,
        edge_c_entity: Entity,
    ) {
        let vertex_a_id = *self.vertex_map.get(&vertex_a_entity).unwrap();
        let vertex_b_id = *self.vertex_map.get(&vertex_b_entity).unwrap();
        let vertex_c_id = *self.vertex_map.get(&vertex_c_entity).unwrap();
        let edge_a_id = *self.edge_map.get(&edge_a_entity).unwrap();
        let edge_b_id = *self.edge_map.get(&edge_b_entity).unwrap();
        let edge_c_id = *self.edge_map.get(&edge_c_entity).unwrap();

        let face_info = IconFrameAction::Face(
            face_index,
            vertex_a_id as u16,
            vertex_b_id as u16,
            vertex_c_id as u16,
            edge_a_id as u16,
            edge_b_id as u16,
            edge_c_id as u16,
        );
        if face_index as usize >= self.face_list.len() {
            self.face_list.resize((face_index + 1) as usize, None);
        }
        self.face_list[face_index as usize] = Some(face_info);
    }

    fn complete_faces(&mut self) {
        let face_list = std::mem::take(&mut self.face_list);
        for face_info_opt in face_list {
            let Some(face_info) = face_info_opt else {
                panic!("face_list contains None");
            };
            self.frame_actions.push(face_info);
        }
    }
}

// Writer
pub struct IconWriter;

impl IconWriter {
    fn world_to_actions(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Vec<IconAction> {
        let working_file_entries = project.working_file_entries();

        let mut actions = Vec::new();

        let mut palette_dependency_key_opt = None;
        let mut bckg_color_entity = None;
        let mut vertex_entities = Vec::new();
        let mut edge_entities = Vec::new();
        let mut face_entities = Vec::new();

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
                ContentEntityData::BackgroundColor(_) => {
                    bckg_color_entity = Some(*content_entity);
                }
                ContentEntityData::IconShape(ShapeType::Vertex) => {
                    vertex_entities.push(*content_entity);
                }
                ContentEntityData::IconShape(ShapeType::Edge) => {
                    edge_entities.push(*content_entity);
                }
                ContentEntityData::IconShape(ShapeType::Face) => {
                    face_entities.push(*content_entity);
                }
                _ => {
                    panic!("icon should not have this content entity type");
                }
            }
        }

        // Write Palette Dependency
        if let Some(dependency_key) = palette_dependency_key_opt {
            info!("writing palette dependency: {}", dependency_key.full_path());
            actions.push(IconAction::PaletteFile(
                dependency_key.path().to_string(),
                dependency_key.name().to_string(),
            ));
        }

        // Write Background Color
        if let Some(bckg_entity) = bckg_color_entity {
            info!("writing background color");
            let mut system_state: SystemState<(
                Server,
                Query<&PaletteColor>,
                Query<&IconBackgroundColor>,
            )> = SystemState::new(world);
            let (server, palette_color_q, bckg_color_q) = system_state.get_mut(world);

            let bckg_color = bckg_color_q.get(bckg_entity).unwrap();

            let palette_entity = bckg_color.palette_color_entity.get(&server).unwrap();
            let palette_color = palette_color_q.get(palette_entity).unwrap();
            let palette_color_index = *palette_color.index;

            actions.push(IconAction::BackgroundColor(palette_color_index));
        }

        let mut frame_map: HashMap<u16, IconFrameActionData> = HashMap::new();

        let mut system_state: SystemState<(
            Server,
            Res<IconManager>,
            Query<&IconVertex>,
            Query<&IconEdge>,
            Query<&IconFace>,
            Query<&IconFrame>,
        )> = SystemState::new(world);
        let (
            server,
            icon_manager,
            vertex_q,
            edge_q,
            face_q,
            frame_q,
        ) = system_state.get_mut(world);

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

            if !frame_map.contains_key(&frame_index) {
                frame_map.insert(frame_index, IconFrameActionData::new());
            }
            let frame_action_data = frame_map.get_mut(&frame_index).unwrap();
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

            if !frame_map.contains_key(&frame_index) {
                frame_map.insert(frame_index, IconFrameActionData::new());
            }
            let frame_action_data = frame_map.get_mut(&frame_index).unwrap();

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
            let Ok(frame) = frame_q.get(frame_entity) else {
                panic!("");
            };
            let frame_index = frame.get_order() as u16;

            if !frame_map.contains_key(&frame_index) {
                frame_map.insert(frame_index, IconFrameActionData::new());
            }
            let frame_action_data = frame_map.get_mut(&frame_index).unwrap();

            let Some(face_index) = icon_manager.get_face_index(&face_entity) else {
                panic!("face entity {:?} does not have an index!", face_entity);
            };

            let vertex_a_entity = face.vertex_a.get(&server).unwrap();
            let vertex_b_entity = face.vertex_b.get(&server).unwrap();
            let vertex_c_entity = face.vertex_c.get(&server).unwrap();
            let edge_a_entity = face.edge_a.get(&server).unwrap();
            let edge_b_entity = face.edge_b.get(&server).unwrap();
            let edge_c_entity = face.edge_c.get(&server).unwrap();

            frame_action_data.add_face(face_index as u16, vertex_a_entity, vertex_b_entity, vertex_c_entity, edge_a_entity, edge_b_entity, edge_c_entity);
        }

        for (_, value) in frame_map.iter_mut() {
            value.complete_faces();
        }

        // Write Frames
        let mut frame_index = 0;

        while frame_map.contains_key(&frame_index) {
            let frame_action_data = frame_map.remove(&frame_index).unwrap();
            actions.push(IconAction::Frame(frame_action_data.frame_actions));
            frame_index += 1;
        }

        if frame_map.len() > 0 {
            panic!("frame_map should be empty!");
        }

        actions
    }

    fn write_from_actions(&self, actions: Vec<IconAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for (action_id, action) in actions.iter().enumerate() {
            match action {
                IconAction::PaletteFile(path, file_name) => {
                    IconActionType::PaletteFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                IconAction::BackgroundColor(palette_color_index) => {
                    IconActionType::BackgroundColor.ser(&mut bit_writer);

                    // TODO: could optimize these a bit more .. unlikely to use all these bits
                    palette_color_index.ser(&mut bit_writer);
                }
                IconAction::Frame(frame_actions) => {

                    let mut test_face_index = 0;

                    for (frame_action_id, frame_action) in frame_actions.iter().enumerate() {

                        IconActionType::Frame.ser(&mut bit_writer);

                        match frame_action {
                            IconFrameAction::Vertex(x, y) => {
                                // continue bit
                                IconFrameActionType::Vertex.ser(&mut bit_writer);

                                // encode X, Y
                                VertexSerdeInt::from(*x).ser(&mut bit_writer);
                                VertexSerdeInt::from(*y).ser(&mut bit_writer);

                                info!("writing vertex {}-{} : ({}, {})", action_id, frame_action_id, x, y);
                            }
                            IconFrameAction::Edge(vertex_a, vertex_b) => {
                                // continue bit
                                IconFrameActionType::Edge.ser(&mut bit_writer);

                                UnsignedVariableInteger::<6>::from(*vertex_a).ser(&mut bit_writer);
                                UnsignedVariableInteger::<6>::from(*vertex_b).ser(&mut bit_writer);

                                info!("writing edge {}-{} : ({}, {})", action_id, frame_action_id, vertex_a, vertex_b);
                            }
                            IconFrameAction::Face(
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
                                IconFrameActionType::Face.ser(&mut bit_writer);

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

                    IconFrameActionType::None.ser(&mut bit_writer);
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
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let actions = self.world_to_actions(world, project, content_entities);
        self.write_from_actions(actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        let mut default_actions = Vec::new();

        let mut frame_actions = Vec::new();
        frame_actions.push(IconFrameAction::Vertex(0, 0));

        default_actions.push(IconAction::Frame(frame_actions));

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

        // read loop
        'outer: loop {
            let action_type = IconActionType::de(bit_reader)?;

            match action_type {
                IconActionType::None => break 'outer,
                IconActionType::PaletteFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    output.push(IconAction::PaletteFile(path, file_name));
                }
                IconActionType::BackgroundColor => {
                    let palette_color_index = u8::de(bit_reader)?;
                    output.push(IconAction::BackgroundColor(palette_color_index));
                }
                IconActionType::Frame => {

                    let mut face_index = 0;

                    let mut frame_output = Vec::new();

                    'inner: loop {
                        let frame_action_type = IconFrameActionType::de(bit_reader)?;

                        match frame_action_type {
                            IconFrameActionType::None => break 'inner,
                            IconFrameActionType::Vertex => {
                                // read X, Y
                                let x = VertexSerdeInt::de(bit_reader)?.to();
                                let y = VertexSerdeInt::de(bit_reader)?.to();

                                frame_output.push(IconFrameAction::Vertex(x, y));
                            }
                            IconFrameActionType::Edge => {
                                let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                                frame_output.push(IconFrameAction::Edge(vertex_a, vertex_b));
                            }
                            IconFrameActionType::Face => {
                                let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let vertex_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                                let edge_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let edge_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let edge_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                                frame_output.push(IconFrameAction::Face(
                                    face_index, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c,
                                ));

                                face_index += 1;
                            }
                        }
                    }

                    output.push(IconAction::Frame(frame_output));
                }
            }
        }
        Ok(output)
    }

    fn actions_to_world(
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        actions: Vec<IconAction>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut system_state: SystemState<(Commands, Server, ResMut<IconManager>)> =
            SystemState::new(world);
        let (mut commands, mut server, mut icon_manager) = system_state.get_mut(world);

        let mut output = Vec::new();
        let mut frame_index = 0;

        for action in actions {
            match action {
                IconAction::PaletteFile(palette_path, palette_file_name) => {
                    let (new_entity, _, new_file_key) = add_file_dependency(
                        project,
                        file_key,
                        file_entity,
                        &mut commands,
                        &mut server,
                        FileExtension::Palette,
                        &palette_path,
                        &palette_file_name,
                    );
                    output.push((new_entity, ContentEntityTypeData::Dependency(new_file_key)));
                }
                IconAction::BackgroundColor(palette_index) => {
                    let mut background_color_component = IconBackgroundColor::new();
                    background_color_component
                        .owning_file_entity
                        .set(&server, file_entity);

                    let entity_id = commands
                        .spawn_empty()
                        .enable_replication(&mut server)
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(background_color_component)
                        .id();
                    info!("spawning background skin color entity {:?}", entity_id);
                    output.push((
                        entity_id,
                        ContentEntityTypeData::BackgroundColor(Some(palette_index)),
                    ));
                }
                IconAction::Frame(frame_actions) => {

                    // make frame entity
                    let mut component = IconFrame::new(frame_index);
                    component.file_entity.set(&server, file_entity);
                    let frame_entity = commands
                        .spawn_empty()
                        .enable_replication(&mut server)
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(component)
                        .id();

                    output.push((frame_entity, ContentEntityTypeData::Frame));

                    icon_manager.on_create_frame(
                        &file_entity,
                        &frame_entity,
                        frame_index as usize,
                        None,
                    );

                    frame_index += 1;

                    // make frame contents
                    let mut vertices = Vec::new();
                    let mut edges = Vec::new();

                    for frame_action in frame_actions {
                        match frame_action {
                            IconFrameAction::Vertex(x, y) => {
                                let mut component = IconVertex::new(x, y);
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
                            IconFrameAction::Edge(vertex_a_index, vertex_b_index) => {
                                let Some(vertex_a_entity) = vertices.get(vertex_a_index as usize) else {
                                    panic!("edge's vertex_a_index is `{:?}` and list of vertices is `{:?}`", vertex_a_index, vertices);
                                };
                                let Some(vertex_b_entity) = vertices.get(vertex_b_index as usize) else {
                                    panic!("edge's vertex_b_index is `{:?}` and list of vertices is `{:?}`", vertex_b_index, vertices);
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
                            IconFrameAction::Face(
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
                                face_component.frame_entity.set(&server, &frame_entity);
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
                                    ContentEntityTypeData::Face(
                                        face_index as usize,
                                        vertex_a_entity,
                                        vertex_b_entity,
                                        vertex_c_entity,
                                    ),
                                ));
                            }
                        }
                    }
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
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = Self::read_to_actions(&mut bit_reader) else {
            panic!("Error reading .icon file");
        };

        let result = Self::actions_to_world(world, project, file_key, file_entity, actions);

        result
    }
}

impl IconReader {

    // TODO: move this into the main read functions
    fn post_process_entities(
        icon_manager: &mut IconManager,
        file_entity: &Entity,
        shape_entities: Vec<(Entity, ContentEntityTypeData)>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut new_content_entities = HashMap::new();

        for (entity, content_entity_type_data) in shape_entities {

            match content_entity_type_data {
                ContentEntityTypeData::Dependency(file_key) => {
                    new_content_entities.insert(
                        entity,
                        ContentEntityData::new_dependency(file_key),
                    );
                }
                ContentEntityTypeData::BackgroundColor(index) => {
                    new_content_entities.insert(
                        entity,
                        ContentEntityData::new_background_color(index),
                    );
                }
                ContentEntityTypeData::Frame => {
                    new_content_entities.insert(
                        entity,
                        ContentEntityData::new_frame(),
                    );
                }
                ContentEntityTypeData::Vertex => {
                    icon_manager.on_create_vertex(entity);

                    new_content_entities.insert(
                        entity,
                        ContentEntityData::new_icon_shape(ShapeType::Vertex),
                    );
                }
                ContentEntityTypeData::Edge(start, end) => {
                    icon_manager.on_create_edge(start, entity, end);

                    new_content_entities.insert(
                        entity,
                        ContentEntityData::new_icon_shape(ShapeType::Edge),
                    );
                }
                ContentEntityTypeData::Face(index, vert_a, vert_b, vert_c) => {
                    icon_manager.on_create_face(
                        file_entity,
                        Some(index),
                        entity,
                        vert_a,
                        vert_b,
                        vert_c,
                    );

                    new_content_entities.insert(
                        entity,
                        ContentEntityData::new_icon_shape(ShapeType::Face),
                    );
                }
            }
        }

        new_content_entities
    }
}
