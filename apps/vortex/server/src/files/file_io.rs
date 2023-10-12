use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, ResMut, SystemState},
    world::World,
};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, ReplicationConfig, Server};

use vortex_proto::{
    components::{EntryKind, FileDependency, FileExtension, FileType, OwnedByFile},
    resources::FileKey,
};

use crate::{
    files::{
        AnimReader, AnimWriter, MeshReader, MeshWriter, PaletteReader, PaletteWriter, SkelReader,
        SkelWriter, SkinReader, SkinWriter,
    },
    resources::{
        AnimationManager, ContentEntityData, PaletteManager, Project, ShapeManager, SkinManager,
    },
};

pub trait FileWriter: Send + Sync {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        content_entities_opt: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]>;
    fn write_new_default(&self) -> Box<[u8]>;
}

pub trait FileReader: Send + Sync {
    fn read(
        &self,
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData>;
}

impl FileReader for FileExtension {
    fn read(
        &self,
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {
        match self {
            FileExtension::Skel => SkelReader.read(world, bytes),
            FileExtension::Mesh => MeshReader.read(world, bytes),
            FileExtension::Anim => AnimReader.read(world, project, file_key, file_entity, bytes),
            FileExtension::Palette => PaletteReader.read(world, file_entity, bytes),
            FileExtension::Skin => SkinReader.read(world, project, file_key, file_entity, bytes),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }
}

impl FileWriter for FileExtension {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        match self {
            FileExtension::Skel => SkelWriter.write(world, project, content_entities),
            FileExtension::Mesh => MeshWriter.write(world, project, content_entities),
            FileExtension::Anim => AnimWriter.write(world, project, content_entities),
            FileExtension::Palette => PaletteWriter.write(world, project, content_entities),
            FileExtension::Skin => SkinWriter.write(world, project, content_entities),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }

    fn write_new_default(&self) -> Box<[u8]> {
        match self {
            FileExtension::Skel => SkelWriter.write_new_default(),
            FileExtension::Mesh => MeshWriter.write_new_default(),
            FileExtension::Anim => AnimWriter.write_new_default(),
            FileExtension::Palette => PaletteWriter.write_new_default(),
            FileExtension::Skin => SkinWriter.write_new_default(),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }
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
    world: &mut World,
    project: &mut Project,
    file_extension: &FileExtension,
    file_key: &FileKey,
    file_entity: &Entity,
    bytes: Box<[u8]>,
) -> HashMap<Entity, ContentEntityData> {
    // FileReader reads File's contents and spawns all Entities + Components
    let new_entities = file_extension.read(world, project, file_key, file_entity, &bytes);

    let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
    let (mut commands, mut server) = system_state.get_mut(world);

    post_process_loaded_networked_entities(
        &mut commands,
        &mut server,
        &new_entities,
        file_entity,
        &file_extension,
    );

    system_state.apply(world);

    new_entities
}

fn post_process_loaded_networked_entities(
    commands: &mut Commands,
    server: &mut Server,
    entities: &HashMap<Entity, ContentEntityData>,
    file_entity: &Entity,
    file_extension: &FileExtension,
) {
    for (entity, data) in entities.iter() {
        match data {
            ContentEntityData::Shape(_) => {
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
            _ => {}
        }
    }
}

pub fn despawn_file_content_entities(
    world: &mut World,
    project: &mut Project,
    file_key: &FileKey,
    content_entities: &HashMap<Entity, ContentEntityData>,
) {
    let mut system_state: SystemState<(
        Commands,
        Server,
        ResMut<ShapeManager>,
        ResMut<AnimationManager>,
        ResMut<PaletteManager>,
        ResMut<SkinManager>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut server,
        mut shape_manager,
        mut animation_manager,
        mut palette_manager,
        mut skin_manager,
    ) = system_state.get_mut(world);

    for (entity, entity_data) in content_entities.iter() {
        info!("despawning entity: {:?}", entity);
        commands
            .entity(*entity)
            .take_authority(&mut server)
            .despawn();

        match entity_data {
            ContentEntityData::Shape(ShapeType::Vertex) => {
                shape_manager.deregister_vertex(entity);
            }
            ContentEntityData::Shape(ShapeType::Edge) => {
                shape_manager.deregister_edge(entity);
            }
            ContentEntityData::Shape(ShapeType::Face) => {}
            ContentEntityData::Dependency(dependency_key) => {
                project.file_remove_dependency(&file_key, &dependency_key);
            }
            ContentEntityData::Frame => {
                animation_manager.deregister_frame(entity, None);
            }
            ContentEntityData::Rotation => {
                animation_manager.deregister_rotation(entity);
            }
            ContentEntityData::PaletteColor => {
                palette_manager.deregister_color(entity, None);
            }
            ContentEntityData::FaceColor => {
                skin_manager.deregister_face_color(entity);
            }
        }
    }

    system_state.apply(world);
}

pub fn add_file_dependency(
    project: &mut Project,
    file_key: &FileKey,
    file_entity: &Entity,
    commands: &mut Commands,
    server: &mut Server,
    dependency_file_ext: FileExtension,
    dependency_path: &str,
    dependency_file_name: &str,
) -> (Entity, FileKey) {
    let dependency_file_key =
        FileKey::new(&dependency_path, &dependency_file_name, EntryKind::File);
    let file_extension = project.file_extension(&dependency_file_key).unwrap();
    if file_extension != dependency_file_ext {
        panic!(
            "new file of type {} is not of the required type: {}",
            file_extension.to_string(),
            dependency_file_ext.to_string()
        );
    }

    project.file_add_dependency(file_key, &dependency_file_key);

    let palette_file_entity = project.file_entity(&dependency_file_key).unwrap();

    info!("creating new FileDependency!");
    let mut component = FileDependency::new();
    component.file_entity.set(server, file_entity);
    component
        .dependency_entity
        .set(server, &palette_file_entity);
    let entity = commands
        .spawn_empty()
        .enable_replication(server)
        .configure_replication(ReplicationConfig::Delegated)
        .insert(component)
        .id();

    return (entity, dependency_file_key);
}
