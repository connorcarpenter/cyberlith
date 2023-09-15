use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Commands, world::World};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, RoomKey, Server};

use vortex_proto::{
    components::{FileExtension, FileType, OwnedByFile},
    resources::FileKey,
};

use crate::{
    files::{AnimReader, AnimWriter, MeshReader, MeshWriter, SkelReader, SkelWriter},
    resources::{ContentEntityData, Project, ShapeManager},
};

pub trait FileWriter: Send + Sync {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        file_key: &FileKey,
        content_entities_opt: &Option<HashMap<Entity, ContentEntityData>>,
    ) -> Box<[u8]>;
    fn write_new_default(&self) -> Box<[u8]>;
}

pub trait FileReader: Send + Sync {
    fn read(
        &self,
        commands: &mut Commands,
        server: &mut Server,
        bytes: &Box<[u8]>,
    ) -> FileReadOutput;
}

impl FileReader for FileExtension {
    fn read(
        &self,
        commands: &mut Commands,
        server: &mut Server,
        bytes: &Box<[u8]>,
    ) -> FileReadOutput {
        match self {
            FileExtension::Skel => SkelReader.read(commands, server, bytes),
            FileExtension::Mesh => MeshReader.read(commands, server, bytes),
            FileExtension::Anim => AnimReader.read(commands, server, bytes),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }
}

impl FileWriter for FileExtension {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        file_key: &FileKey,
        content_entities_opt: &Option<HashMap<Entity, ContentEntityData>>,
    ) -> Box<[u8]> {
        match self {
            FileExtension::Skel => SkelWriter.write(world, project, file_key, content_entities_opt),
            FileExtension::Mesh => MeshWriter.write(world, project, file_key, content_entities_opt),
            FileExtension::Anim => AnimWriter.write(world, project, file_key, content_entities_opt),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }

    fn write_new_default(&self) -> Box<[u8]> {
        match self {
            FileExtension::Skel => SkelWriter.write_new_default(),
            FileExtension::Mesh => MeshWriter.write_new_default(),
            FileExtension::Anim => AnimWriter.write_new_default(),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }
}

pub enum FileReadOutput {
    // Skel file, list of (vertex 3d entity, and an optional (edge 3d entity, parent vertex 3d entity))
    Skel(Vec<(Entity, Option<(Entity, Entity)>)>),
    // Mesh file, list of vert/edge/face entities
    Mesh(Vec<(Entity, ShapeTypeData)>),
    // Option<(SkelPath, SkelFile)>
    Anim(Option<(String, String)>),
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum ShapeType {
    Vertex,
    Edge,
    Face,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum ShapeTypeData {
    Vertex,
    Edge(Entity, Entity),
    Face(Entity, Entity, Entity, Entity, Entity, Entity),
}

impl From<ShapeTypeData> for ShapeType {
    fn from(shape_type_data: ShapeTypeData) -> Self {
        match shape_type_data {
            ShapeTypeData::Vertex => ShapeType::Vertex,
            ShapeTypeData::Edge(_, _) => ShapeType::Edge,
            ShapeTypeData::Face(_, _, _, _, _, _) => ShapeType::Face,
        }
    }
}

pub fn load_content_entities(
    commands: &mut Commands,
    server: &mut Server,
    project: &mut Project,
    shape_manager: &mut ShapeManager,
    file_extension: &FileExtension,
    file_room_key: &RoomKey,
    file_key: &FileKey,
    file_entity: &Entity,
    bytes: Box<[u8]>,
) -> HashMap<Entity, ContentEntityData> {
    // FileReader reads File's contents and spawns all Entities + Components
    let read_output = file_extension.read(commands, server, &bytes);

    let new_entities = match read_output {
        FileReadOutput::Skel(entities) => {
            SkelReader::post_process_entities(shape_manager, entities)
        }
        FileReadOutput::Mesh(shape_entities) => {
            MeshReader::post_process_entities(shape_manager, shape_entities)
        }
        FileReadOutput::Anim(skel_path_opt) => AnimReader::post_process(
            commands,
            server,
            project,
            file_key,
            file_entity,
            skel_path_opt,
        ),
    };

    post_process_loaded_networked_entities(
        commands,
        server,
        file_room_key,
        &new_entities,
        file_entity,
        &file_extension,
    );

    new_entities
}

fn post_process_loaded_networked_entities(
    commands: &mut Commands,
    server: &mut Server,
    room_key: &RoomKey,
    entities: &HashMap<Entity, ContentEntityData>,
    file_entity: &Entity,
    file_extension: &FileExtension,
) {
    for (entity, _data) in entities.iter() {
        // associate all new Entities with the new Room
        server.room_mut(room_key).add_entity(entity);

        // add file ownership
        let mut file_ownership_component = OwnedByFile::new();
        file_ownership_component
            .file_entity
            .set(server, file_entity);
        commands.entity(*entity).insert(file_ownership_component);

        // add FileType component
        match file_extension {
            FileExtension::Skel => {
                commands
                    .entity(*entity)
                    .insert(FileType::new(FileExtension::Skel));
            }
            FileExtension::Mesh => {
                commands
                    .entity(*entity)
                    .insert(FileType::new(FileExtension::Mesh));
            }
            FileExtension::Anim => {
                commands
                    .entity(*entity)
                    .insert(FileType::new(FileExtension::Anim));
            }
            _ => panic!("File extension {:?} not implemented", file_extension),
        }
    }
}

pub fn despawn_file_content_entities(
    commands: &mut Commands,
    server: &mut Server,
    shape_manager: &mut ShapeManager,
    project: &mut Project,
    file_key: &FileKey,
    content_entities: &HashMap<Entity, ContentEntityData>,
) {
    for (entity, entity_data) in content_entities.iter() {
        info!("despawning entity: {:?}", entity);
        commands.entity(*entity).take_authority(server).despawn();

        match entity_data {
            ContentEntityData::Shape(ShapeType::Vertex) => {
                shape_manager.on_delete_vertex(commands, server, entity);
            }
            ContentEntityData::Shape(ShapeType::Edge) => {
                shape_manager.on_delete_edge(entity);
            }
            ContentEntityData::Shape(ShapeType::Face) => {}
            ContentEntityData::Dependency(dependency_key) => {
                project.file_remove_dependency(&file_key, &dependency_key);
            }
        }
    }
}
