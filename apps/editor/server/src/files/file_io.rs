use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, ResMut, SystemState},
    world::World,
};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, ReplicationConfig, Server};

use asset_io::json::{AnimFileQuat, AssetId};
use math::Quat;

use editor_proto::{
    components::{FileDependency, FileExtension, FileType, OwnedByFile},
    resources::FileKey,
};

use crate::{
    files::{
        AnimReader, AnimWriter, IconReader, IconWriter, MeshReader, MeshWriter, ModelReader,
        ModelWriter, PaletteReader, PaletteWriter, SceneReader, SceneWriter, SkelReader,
        SkelWriter, SkinReader, SkinWriter,
    },
    resources::{
        AnimationManager, ContentEntityData, IconManager, PaletteManager, Project, ShapeManager,
        SkinManager,
    },
};

pub trait FileWriter: Send + Sync {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        content_entities_opt: &HashMap<Entity, ContentEntityData>,
        asset_id: &AssetId,
    ) -> Box<[u8]>;
    fn write_new_default(
        &self,
        asset_id: &AssetId,
    ) -> Box<[u8]>;
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
            FileExtension::Mesh => MeshReader.read(world, file_entity, bytes),
            FileExtension::Anim => AnimReader.read(world, project, file_key, file_entity, bytes),
            FileExtension::Palette => PaletteReader.read(world, file_entity, bytes),
            FileExtension::Skin => SkinReader.read(world, project, file_key, file_entity, bytes),
            FileExtension::Model => ModelReader.read(world, project, file_key, file_entity, bytes),
            FileExtension::Scene => SceneReader.read(world, project, file_key, file_entity, bytes),
            FileExtension::Icon => IconReader.read(world, project, file_key, file_entity, bytes),
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
        asset_id: &AssetId,
    ) -> Box<[u8]> {
        match self {
            FileExtension::Skel => SkelWriter.write(world, project, content_entities, asset_id),
            FileExtension::Mesh => MeshWriter.write(world, project, content_entities, asset_id),
            FileExtension::Anim => AnimWriter.write(world, project, content_entities, asset_id),
            FileExtension::Palette => PaletteWriter.write(world, project, content_entities, asset_id),
            FileExtension::Skin => SkinWriter.write(world, project, content_entities, asset_id),
            FileExtension::Model => ModelWriter.write(world, project, content_entities, asset_id),
            FileExtension::Scene => SceneWriter.write(world, project, content_entities, asset_id),
            FileExtension::Icon => IconWriter.write(world, project, content_entities, asset_id),
            _ => panic!("File extension {:?} not implemented", self),
        }
    }

    fn write_new_default(&self, asset_id: &AssetId) -> Box<[u8]> {
        match self {
            FileExtension::Skel => SkelWriter.write_new_default(asset_id),
            FileExtension::Mesh => MeshWriter.write_new_default(asset_id),
            FileExtension::Anim => AnimWriter.write_new_default(asset_id),
            FileExtension::Palette => PaletteWriter.write_new_default(asset_id),
            FileExtension::Skin => SkinWriter.write_new_default(asset_id),
            FileExtension::Model => ModelWriter.write_new_default(asset_id),
            FileExtension::Scene => SceneWriter.write_new_default(asset_id),
            FileExtension::Icon => IconWriter.write_new_default(asset_id),
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
    Face(usize, Entity, Entity, Entity),
}

impl From<ShapeTypeData> for ShapeType {
    fn from(shape_type_data: ShapeTypeData) -> Self {
        match shape_type_data {
            ShapeTypeData::Vertex => ShapeType::Vertex,
            ShapeTypeData::Edge(_, _) => ShapeType::Edge,
            ShapeTypeData::Face(_, _, _, _) => ShapeType::Face,
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

    // TODO: handle this in initial read
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

// TODO: remove this and handle in initial read
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
            ContentEntityData::IconShape(_) | ContentEntityData::IconFace(_) => {
                // add file ownership
                let mut file_ownership_component = OwnedByFile::new();
                file_ownership_component
                    .file_entity
                    .set(server, file_entity);
                commands.entity(*entity).insert(file_ownership_component);
            }
            _ => {}
        }
    }
}

pub fn despawn_file_content_entities(
    world: &mut World,
    project: &mut Project,
    file_ext: &FileExtension,
    file_key: &FileKey,
    content_entities: &HashMap<Entity, ContentEntityData>,
) {
    let mut system_state: SystemState<(
        Commands,
        Server,
        ResMut<ShapeManager>,
        ResMut<IconManager>,
        ResMut<AnimationManager>,
        ResMut<PaletteManager>,
        ResMut<SkinManager>,
    )> = SystemState::new(world);
    let (
        mut commands,
        mut server,
        mut shape_manager,
        mut icon_manager,
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

        match (file_ext, entity_data) {
            (_, ContentEntityData::Dependency(dependency_key)) => {
                project.file_remove_dependency(&file_key, &dependency_key);
            }
            (_, ContentEntityData::Shape(ShapeType::Vertex)) => {
                shape_manager.deregister_vertex(entity);
            }
            (_, ContentEntityData::Shape(ShapeType::Edge)) => {
                shape_manager.deregister_edge(entity);
            }
            (_, ContentEntityData::Shape(ShapeType::Face)) => {
                shape_manager.deregister_face(entity);
            }
            (FileExtension::Icon, ContentEntityData::IconShape(ShapeType::Vertex)) => {
                icon_manager.deregister_vertex(entity);
            }
            (FileExtension::Icon, ContentEntityData::IconShape(ShapeType::Edge)) => {
                icon_manager.deregister_edge(entity);
            }
            (FileExtension::Icon, ContentEntityData::IconShape(ShapeType::Face)) => {
                panic!("incorrect data type");
            }
            (FileExtension::Icon, ContentEntityData::IconFace(_palette_color_opt)) => {
                icon_manager.deregister_face(entity);
            }
            (FileExtension::Icon, ContentEntityData::Frame) => {
                icon_manager.deregister_frame(entity, None);
            }
            (FileExtension::Anim, ContentEntityData::Frame) => {
                animation_manager.deregister_frame(entity, None);
            }
            (FileExtension::Anim, ContentEntityData::Rotation) => {
                animation_manager.deregister_rotation(entity);
            }
            (FileExtension::Palette, ContentEntityData::PaletteColor) => {
                palette_manager.deregister_color(entity, None);
            }
            (FileExtension::Skin, ContentEntityData::BackgroundColor(_)) => {
                // deregister with skin_manager?
            }
            (FileExtension::Skin, ContentEntityData::FaceColor(_)) => {
                skin_manager.deregister_face_color(entity);
            }
            (FileExtension::Model | FileExtension::Scene, ContentEntityData::NetTransform) => {
                // deregister with model_manager?
            }
            (_, _) => {
                panic!(
                    "unknown content entity type! file ext: {:?}, data: {:?}",
                    file_ext, entity_data
                );
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
    dependency_file_key: &FileKey,
) -> (Entity, Entity) {
    let file_extension = project.file_extension(dependency_file_key).unwrap();
    if file_extension != dependency_file_ext {
        panic!(
            "new file of type {} is not of the required type: {}",
            file_extension.to_string(),
            dependency_file_ext.to_string()
        );
    }

    project.file_add_dependency(file_key, dependency_file_key);

    let dependency_file_entity = project.file_entity(dependency_file_key).unwrap();

    info!("creating new FileDependency!");
    let mut component = FileDependency::new();
    component.file_entity.set(server, file_entity);
    component
        .dependency_entity
        .set(server, &dependency_file_entity);
    let dependency_entity = commands
        .spawn_empty()
        .enable_replication(server)
        .configure_replication(ReplicationConfig::Delegated)
        .insert(component)
        .id();

    return (
        dependency_entity,
        dependency_file_entity,
    );
}

// conversion

// transition

// quat
pub fn convert_into_quat(input: editor_proto::SerdeQuat) -> AnimFileQuat {
    let quat: Quat = input.into();
    AnimFileQuat::from_xyzw(quat.x, quat.y, quat.z, quat.w)
}

pub fn convert_from_quat(input: &AnimFileQuat) -> editor_proto::SerdeQuat {
    let quat = Quat::from_xyzw(input.get_x(), input.get_y(), input.get_z(), input.get_w());
    editor_proto::SerdeQuat::from(quat)
}

// pub fn convert_into_quat_map(
//     input: HashMap<u16, editor_proto::SerdeQuat>,
// ) -> HashMap<u16, AnimFileQuat> {
//     let mut output = HashMap::new();
//     for (key, value) in input.iter() {
//         let value = AnimFileQuat::from_xyzw(value.0.x, value.0.y, value.0.z, value.0.w);
//         output.insert(*key, value);
//     }
//     output
// }

// rotation
pub fn convert_into_rotation(
    input: editor_proto::components::SerdeRotation,
) -> u8 {
    input.to_inner()
}

pub fn convert_from_rotation(
    input: u8,
) -> editor_proto::components::SerdeRotation {
    editor_proto::components::SerdeRotation::from_inner(input)
}

// transform type
pub fn convert_into_transform_type(
    input: editor_proto::components::NetTransformEntityType,
) -> String {
    match input {
        editor_proto::components::NetTransformEntityType::Skin => {
            "skin".to_string()
        }
        editor_proto::components::NetTransformEntityType::Scene => {
            "scene".to_string()
        }
        _ => {
            panic!("unsupported");
        }
    }
}

pub fn convert_from_transform_type(
    input: &str
) -> editor_proto::components::NetTransformEntityType {
    match input {
        "skin" => {
            editor_proto::components::NetTransformEntityType::Skin
        }
        "scene" => {
            editor_proto::components::NetTransformEntityType::Scene
        }
        _ => {
            panic!("not supported");
        }
    }
}
